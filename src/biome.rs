#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BiomeKey {
    pub id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClimateParameter {
    pub min: i64,
    pub max: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClimateTarget {
    pub temperature: i64,
    pub humidity: i64,
    pub continentalness: i64,
    pub erosion: i64,
    pub depth: i64,
    pub weirdness: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClimateParameterPoint {
    pub temperature: ClimateParameter,
    pub humidity: ClimateParameter,
    pub continentalness: ClimateParameter,
    pub erosion: ClimateParameter,
    pub depth: ClimateParameter,
    pub weirdness: ClimateParameter,
    pub offset: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClimateBiomeEntry {
    pub parameters: ClimateParameterPoint,
    pub biome: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultiNoiseBiomeSourceParameterListPreset {
    pub id: &'static str,
    pub parameters: &'static [ClimateBiomeEntry],
    pub used_biomes: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClimateSamplerSample {
    pub quart_x: i32,
    pub quart_y: i32,
    pub quart_z: i32,
    pub block_x: i32,
    pub block_y: i32,
    pub block_z: i32,
    pub target: ClimateTarget,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClimateSamplerInput {
    pub temperature: f32,
    pub humidity: f32,
    pub continentalness: f32,
    pub erosion: f32,
    pub depth: f32,
    pub weirdness: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClimateParameterList {
    pub values: Vec<ClimateBiomeEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClimateRTreeNode {
    pub parameter_space: [ClimateParameter; 7],
    pub value_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BiomeSourceModel {
    Fixed {
        biome: &'static str,
    },
    Checkerboard {
        biomes: Vec<&'static str>,
        scale: i32,
    },
    MultiNoisePreset {
        preset: &'static str,
    },
    TheEnd,
}

pub const QUANTIZATION_FACTOR: f32 = 10_000.0;

// Source: decompiled-server-26.1.2/net/minecraft/world/level/biome/Biome.java
pub const BUILTIN_BIOMES: &[BiomeKey] = &[
    BiomeKey {
        id: "minecraft:the_void",
    },
    BiomeKey {
        id: "minecraft:plains",
    },
    BiomeKey {
        id: "minecraft:sunflower_plains",
    },
    BiomeKey {
        id: "minecraft:snowy_plains",
    },
    BiomeKey {
        id: "minecraft:ice_spikes",
    },
    BiomeKey {
        id: "minecraft:desert",
    },
    BiomeKey {
        id: "minecraft:swamp",
    },
    BiomeKey {
        id: "minecraft:mangrove_swamp",
    },
    BiomeKey {
        id: "minecraft:forest",
    },
    BiomeKey {
        id: "minecraft:flower_forest",
    },
    BiomeKey {
        id: "minecraft:birch_forest",
    },
    BiomeKey {
        id: "minecraft:dark_forest",
    },
    BiomeKey {
        id: "minecraft:pale_garden",
    },
    BiomeKey {
        id: "minecraft:old_growth_birch_forest",
    },
    BiomeKey {
        id: "minecraft:old_growth_pine_taiga",
    },
    BiomeKey {
        id: "minecraft:old_growth_spruce_taiga",
    },
    BiomeKey {
        id: "minecraft:taiga",
    },
    BiomeKey {
        id: "minecraft:snowy_taiga",
    },
    BiomeKey {
        id: "minecraft:savanna",
    },
    BiomeKey {
        id: "minecraft:savanna_plateau",
    },
    BiomeKey {
        id: "minecraft:windswept_hills",
    },
    BiomeKey {
        id: "minecraft:windswept_gravelly_hills",
    },
    BiomeKey {
        id: "minecraft:windswept_forest",
    },
    BiomeKey {
        id: "minecraft:windswept_savanna",
    },
    BiomeKey {
        id: "minecraft:jungle",
    },
    BiomeKey {
        id: "minecraft:sparse_jungle",
    },
    BiomeKey {
        id: "minecraft:bamboo_jungle",
    },
    BiomeKey {
        id: "minecraft:badlands",
    },
    BiomeKey {
        id: "minecraft:eroded_badlands",
    },
    BiomeKey {
        id: "minecraft:wooded_badlands",
    },
    BiomeKey {
        id: "minecraft:meadow",
    },
    BiomeKey {
        id: "minecraft:cherry_grove",
    },
    BiomeKey {
        id: "minecraft:grove",
    },
    BiomeKey {
        id: "minecraft:snowy_slopes",
    },
    BiomeKey {
        id: "minecraft:frozen_peaks",
    },
    BiomeKey {
        id: "minecraft:jagged_peaks",
    },
    BiomeKey {
        id: "minecraft:stony_peaks",
    },
    BiomeKey {
        id: "minecraft:river",
    },
    BiomeKey {
        id: "minecraft:frozen_river",
    },
    BiomeKey {
        id: "minecraft:beach",
    },
    BiomeKey {
        id: "minecraft:snowy_beach",
    },
    BiomeKey {
        id: "minecraft:stony_shore",
    },
    BiomeKey {
        id: "minecraft:warm_ocean",
    },
    BiomeKey {
        id: "minecraft:lukewarm_ocean",
    },
    BiomeKey {
        id: "minecraft:deep_lukewarm_ocean",
    },
    BiomeKey {
        id: "minecraft:ocean",
    },
    BiomeKey {
        id: "minecraft:deep_ocean",
    },
    BiomeKey {
        id: "minecraft:cold_ocean",
    },
    BiomeKey {
        id: "minecraft:deep_cold_ocean",
    },
    BiomeKey {
        id: "minecraft:frozen_ocean",
    },
    BiomeKey {
        id: "minecraft:deep_frozen_ocean",
    },
    BiomeKey {
        id: "minecraft:mushroom_fields",
    },
    BiomeKey {
        id: "minecraft:dripstone_caves",
    },
    BiomeKey {
        id: "minecraft:lush_caves",
    },
    BiomeKey {
        id: "minecraft:deep_dark",
    },
    BiomeKey {
        id: "minecraft:nether_wastes",
    },
    BiomeKey {
        id: "minecraft:warped_forest",
    },
    BiomeKey {
        id: "minecraft:crimson_forest",
    },
    BiomeKey {
        id: "minecraft:soul_sand_valley",
    },
    BiomeKey {
        id: "minecraft:basalt_deltas",
    },
    BiomeKey {
        id: "minecraft:the_end",
    },
    BiomeKey {
        id: "minecraft:end_highlands",
    },
    BiomeKey {
        id: "minecraft:end_midlands",
    },
    BiomeKey {
        id: "minecraft:small_end_islands",
    },
    BiomeKey {
        id: "minecraft:end_barrens",
    },
];

pub const NETHER_BIOME_PARAMETERS: &[ClimateBiomeEntry] = &[
    ClimateBiomeEntry {
        parameters: climate_point(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        biome: "minecraft:nether_wastes",
    },
    ClimateBiomeEntry {
        parameters: climate_point(0.0, -0.5, 0.0, 0.0, 0.0, 0.0, 0.0),
        biome: "minecraft:soul_sand_valley",
    },
    ClimateBiomeEntry {
        parameters: climate_point(0.4, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        biome: "minecraft:crimson_forest",
    },
    ClimateBiomeEntry {
        parameters: climate_point(0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.375),
        biome: "minecraft:warped_forest",
    },
    ClimateBiomeEntry {
        parameters: climate_point(-0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.175),
        biome: "minecraft:basalt_deltas",
    },
];

pub const OVERWORLD_BIOME_PARAMETERS: &[ClimateBiomeEntry] = &[
    ClimateBiomeEntry {
        parameters: ClimateParameterPoint {
            temperature: span(-1.0, 1.0),
            humidity: span(-1.0, 1.0),
            continentalness: span(-1.2, -1.05),
            erosion: span(-1.0, 1.0),
            depth: point(0.0),
            weirdness: span(-1.0, 1.0),
            offset: 0,
        },
        biome: "minecraft:mushroom_fields",
    },
    ClimateBiomeEntry {
        parameters: ClimateParameterPoint {
            temperature: span(-1.0, -0.45),
            humidity: span(-1.0, 1.0),
            continentalness: span(-1.05, -0.455),
            erosion: span(-1.0, 1.0),
            depth: point(0.0),
            weirdness: span(-1.0, 1.0),
            offset: 0,
        },
        biome: "minecraft:deep_frozen_ocean",
    },
    ClimateBiomeEntry {
        parameters: ClimateParameterPoint {
            temperature: span(-0.45, -0.15),
            humidity: span(-1.0, 1.0),
            continentalness: span(-1.05, -0.455),
            erosion: span(-1.0, 1.0),
            depth: point(0.0),
            weirdness: span(-1.0, 1.0),
            offset: 0,
        },
        biome: "minecraft:deep_cold_ocean",
    },
    ClimateBiomeEntry {
        parameters: ClimateParameterPoint {
            temperature: span(-0.15, 0.2),
            humidity: span(-1.0, 1.0),
            continentalness: span(-1.05, -0.455),
            erosion: span(-1.0, 1.0),
            depth: point(0.0),
            weirdness: span(-1.0, 1.0),
            offset: 0,
        },
        biome: "minecraft:deep_ocean",
    },
    ClimateBiomeEntry {
        parameters: ClimateParameterPoint {
            temperature: span(0.2, 0.55),
            humidity: span(-1.0, 1.0),
            continentalness: span(-0.455, -0.19),
            erosion: span(-1.0, 1.0),
            depth: point(0.0),
            weirdness: span(-1.0, 1.0),
            offset: 0,
        },
        biome: "minecraft:lukewarm_ocean",
    },
    ClimateBiomeEntry {
        parameters: ClimateParameterPoint {
            temperature: span(0.55, 1.0),
            humidity: span(-1.0, 1.0),
            continentalness: span(-0.455, -0.19),
            erosion: span(-1.0, 1.0),
            depth: point(0.0),
            weirdness: span(-1.0, 1.0),
            offset: 0,
        },
        biome: "minecraft:warm_ocean",
    },
    ClimateBiomeEntry {
        parameters: ClimateParameterPoint {
            temperature: span(-1.0, -0.45),
            humidity: span(-1.0, 1.0),
            continentalness: span(-0.11, 0.55),
            erosion: span(-1.0, 1.0),
            depth: point(0.0),
            weirdness: span(-1.0, 1.0),
            offset: 0,
        },
        biome: "minecraft:snowy_plains",
    },
    ClimateBiomeEntry {
        parameters: ClimateParameterPoint {
            temperature: span(-0.45, -0.15),
            humidity: span(-1.0, -0.35),
            continentalness: span(-0.11, 0.55),
            erosion: span(-1.0, 1.0),
            depth: point(0.0),
            weirdness: span(-1.0, 1.0),
            offset: 0,
        },
        biome: "minecraft:plains",
    },
    ClimateBiomeEntry {
        parameters: ClimateParameterPoint {
            temperature: span(-0.15, 0.2),
            humidity: span(-0.1, 0.1),
            continentalness: span(-0.11, 0.55),
            erosion: span(-1.0, 1.0),
            depth: point(0.0),
            weirdness: span(-1.0, 1.0),
            offset: 0,
        },
        biome: "minecraft:forest",
    },
    ClimateBiomeEntry {
        parameters: ClimateParameterPoint {
            temperature: span(0.2, 0.55),
            humidity: span(-1.0, -0.35),
            continentalness: span(0.3, 1.0),
            erosion: span(-1.0, 1.0),
            depth: point(0.0),
            weirdness: span(-1.0, 1.0),
            offset: 0,
        },
        biome: "minecraft:savanna",
    },
    ClimateBiomeEntry {
        parameters: ClimateParameterPoint {
            temperature: span(0.55, 1.0),
            humidity: span(-1.0, 1.0),
            continentalness: span(-0.11, 1.0),
            erosion: span(-1.0, 1.0),
            depth: point(0.0),
            weirdness: span(-1.0, 1.0),
            offset: 0,
        },
        biome: "minecraft:desert",
    },
    ClimateBiomeEntry {
        parameters: ClimateParameterPoint {
            temperature: span(-1.0, 1.0),
            humidity: span(-1.0, 1.0),
            continentalness: span(-1.0, 1.0),
            erosion: span(-1.0, 1.0),
            depth: span(0.2, 0.9),
            weirdness: span(-1.0, 1.0),
            offset: 0,
        },
        biome: "minecraft:dripstone_caves",
    },
    ClimateBiomeEntry {
        parameters: ClimateParameterPoint {
            temperature: span(-1.0, 1.0),
            humidity: span(0.3, 1.0),
            continentalness: span(-1.0, 1.0),
            erosion: span(-1.0, 1.0),
            depth: span(0.2, 0.9),
            weirdness: span(-1.0, 1.0),
            offset: 0,
        },
        biome: "minecraft:lush_caves",
    },
    ClimateBiomeEntry {
        parameters: ClimateParameterPoint {
            temperature: span(-1.0, 1.0),
            humidity: span(-1.0, 1.0),
            continentalness: span(-1.0, 1.0),
            erosion: span(-1.0, -0.225),
            depth: span(0.9, 1.0),
            weirdness: span(-1.0, 1.0),
            offset: 0,
        },
        biome: "minecraft:deep_dark",
    },
];

pub const NETHER_MULTI_NOISE_USED_BIOMES: &[&str] = &[
    "minecraft:nether_wastes",
    "minecraft:soul_sand_valley",
    "minecraft:crimson_forest",
    "minecraft:warped_forest",
    "minecraft:basalt_deltas",
];

pub const OVERWORLD_MULTI_NOISE_USED_BIOMES: &[&str] = &[
    "minecraft:mushroom_fields",
    "minecraft:deep_frozen_ocean",
    "minecraft:deep_cold_ocean",
    "minecraft:deep_ocean",
    "minecraft:lukewarm_ocean",
    "minecraft:warm_ocean",
    "minecraft:snowy_plains",
    "minecraft:plains",
    "minecraft:forest",
    "minecraft:savanna",
    "minecraft:desert",
    "minecraft:dripstone_caves",
    "minecraft:lush_caves",
    "minecraft:deep_dark",
];

pub const fn quantize_coord(coord: f32) -> i64 {
    (coord * QUANTIZATION_FACTOR) as i64
}

pub const fn quart_to_block(coord: i32) -> i32 {
    coord * 4
}

pub fn unquantize_coord(coord: i64) -> f32 {
    coord as f32 / QUANTIZATION_FACTOR
}

pub const fn climate_sampler_sample(
    quart_x: i32,
    quart_y: i32,
    quart_z: i32,
    input: ClimateSamplerInput,
) -> ClimateSamplerSample {
    ClimateSamplerSample {
        quart_x,
        quart_y,
        quart_z,
        block_x: quart_to_block(quart_x),
        block_y: quart_to_block(quart_y),
        block_z: quart_to_block(quart_z),
        target: climate_target(
            input.temperature,
            input.humidity,
            input.continentalness,
            input.erosion,
            input.depth,
            input.weirdness,
        ),
    }
}

pub const fn climate_target(
    temperature: f32,
    humidity: f32,
    continentalness: f32,
    erosion: f32,
    depth: f32,
    weirdness: f32,
) -> ClimateTarget {
    ClimateTarget {
        temperature: quantize_coord(temperature),
        humidity: quantize_coord(humidity),
        continentalness: quantize_coord(continentalness),
        erosion: quantize_coord(erosion),
        depth: quantize_coord(depth),
        weirdness: quantize_coord(weirdness),
    }
}

pub const fn point(value: f32) -> ClimateParameter {
    span(value, value)
}

pub const fn span(min: f32, max: f32) -> ClimateParameter {
    ClimateParameter {
        min: quantize_coord(min),
        max: quantize_coord(max),
    }
}

pub const fn climate_point(
    temperature: f32,
    humidity: f32,
    continentalness: f32,
    erosion: f32,
    depth: f32,
    weirdness: f32,
    offset: f32,
) -> ClimateParameterPoint {
    ClimateParameterPoint {
        temperature: point(temperature),
        humidity: point(humidity),
        continentalness: point(continentalness),
        erosion: point(erosion),
        depth: point(depth),
        weirdness: point(weirdness),
        offset: quantize_coord(offset),
    }
}

impl ClimateParameter {
    pub fn distance(self, target: i64) -> i64 {
        let above = target - self.max;
        let below = self.min - target;
        if above > 0 {
            above
        } else {
            below.max(0)
        }
    }

    pub fn distance_parameter(self, target: ClimateParameter) -> i64 {
        let above = target.min - self.max;
        let below = self.min - target.max;
        if above > 0 {
            above
        } else {
            below.max(0)
        }
    }

    pub fn span_parameter(self, other: Option<ClimateParameter>) -> ClimateParameter {
        other.map_or(self, |other| ClimateParameter {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        })
    }
}

impl ClimateParameterPoint {
    pub fn fitness(self, target: ClimateTarget) -> i64 {
        square(self.temperature.distance(target.temperature))
            + square(self.humidity.distance(target.humidity))
            + square(self.continentalness.distance(target.continentalness))
            + square(self.erosion.distance(target.erosion))
            + square(self.depth.distance(target.depth))
            + square(self.weirdness.distance(target.weirdness))
            + square(self.offset)
    }

    pub fn parameter_space(self) -> [ClimateParameter; 7] {
        [
            self.temperature,
            self.humidity,
            self.continentalness,
            self.erosion,
            self.depth,
            self.weirdness,
            ClimateParameter {
                min: self.offset,
                max: self.offset,
            },
        ]
    }
}

pub fn select_climate_biome(
    entries: &[ClimateBiomeEntry],
    target: ClimateTarget,
) -> Option<&'static str> {
    entries
        .iter()
        .min_by_key(|entry| entry.parameters.fitness(target))
        .map(|entry| entry.biome)
}

impl ClimateTarget {
    pub fn parameter_array(self) -> [i64; 7] {
        [
            self.temperature,
            self.humidity,
            self.continentalness,
            self.erosion,
            self.depth,
            self.weirdness,
            0,
        ]
    }
}

impl ClimateParameterList {
    pub fn new(values: Vec<ClimateBiomeEntry>) -> Result<Self, String> {
        if values.is_empty() {
            return Err("Need at least one value to build the search tree.".to_string());
        }
        Ok(Self { values })
    }

    pub fn find_value_bruteforce(&self, target: ClimateTarget) -> &'static str {
        self.values
            .iter()
            .min_by_key(|entry| entry.parameters.fitness(target))
            .expect("parameter list is non-empty")
            .biome
    }

    pub fn find_value_index(&self, target: ClimateTarget) -> &'static str {
        self.rtree_nodes()
            .into_iter()
            .min_by_key(|node| {
                climate_node_distance(node.parameter_space, target.parameter_array())
            })
            .and_then(|node| self.values.get(node.value_index))
            .expect("parameter list is non-empty")
            .biome
    }

    pub fn rtree_nodes(&self) -> Vec<ClimateRTreeNode> {
        let mut nodes = self
            .values
            .iter()
            .enumerate()
            .map(|(value_index, entry)| ClimateRTreeNode {
                parameter_space: entry.parameters.parameter_space(),
                value_index,
            })
            .collect::<Vec<_>>();
        nodes.sort_by_key(|node| {
            node.parameter_space
                .iter()
                .map(|parameter| ((parameter.min + parameter.max) / 2).abs())
                .sum::<i64>()
        });
        nodes
    }
}

pub fn climate_node_distance(parameter_space: [ClimateParameter; 7], target: [i64; 7]) -> i64 {
    parameter_space
        .into_iter()
        .zip(target)
        .map(|(parameter, target)| square(parameter.distance(target)))
        .sum()
}

pub fn biome_source_codec(id: &str) -> Option<&'static str> {
    match id.strip_prefix("minecraft:").unwrap_or(id) {
        "fixed" => Some("minecraft:fixed"),
        "multi_noise" => Some("minecraft:multi_noise"),
        "checkerboard" => Some("minecraft:checkerboard"),
        "the_end" => Some("minecraft:the_end"),
        _ => None,
    }
}

pub fn biome_source_from_stem_id(id: &str) -> Option<BiomeSourceModel> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    match name {
        "plains" => Some(BiomeSourceModel::Fixed {
            biome: "minecraft:plains",
        }),
        "fixed/plains" => Some(BiomeSourceModel::Fixed {
            biome: "minecraft:plains",
        }),
        "multi_noise/nether" => Some(BiomeSourceModel::MultiNoisePreset {
            preset: "minecraft:nether",
        }),
        "multi_noise/overworld" => Some(BiomeSourceModel::MultiNoisePreset {
            preset: "minecraft:overworld",
        }),
        "the_end" => Some(BiomeSourceModel::TheEnd),
        _ => None,
    }
}

pub fn multi_noise_parameter_list_preset(
    id: &str,
) -> Option<MultiNoiseBiomeSourceParameterListPreset> {
    match id.strip_prefix("minecraft:").unwrap_or(id) {
        "nether" => Some(MultiNoiseBiomeSourceParameterListPreset {
            id: "minecraft:nether",
            parameters: NETHER_BIOME_PARAMETERS,
            used_biomes: NETHER_MULTI_NOISE_USED_BIOMES,
        }),
        "overworld" => Some(MultiNoiseBiomeSourceParameterListPreset {
            id: "minecraft:overworld",
            parameters: OVERWORLD_BIOME_PARAMETERS,
            used_biomes: OVERWORLD_MULTI_NOISE_USED_BIOMES,
        }),
        _ => None,
    }
}

pub fn checkerboard_biome_source(
    biomes: Vec<&'static str>,
    scale: i32,
) -> Result<BiomeSourceModel, String> {
    if !(0..=62).contains(&scale) {
        return Err("Checkerboard biome source scale must be in 0..=62".to_string());
    }
    if biomes.is_empty() {
        return Err("Checkerboard biome source requires at least one biome".to_string());
    }
    Ok(BiomeSourceModel::Checkerboard { biomes, scale })
}

pub fn select_biome_from_source(
    source: &BiomeSourceModel,
    quart_x: i32,
    quart_y: i32,
    quart_z: i32,
    climate: ClimateTarget,
    end_erosion: f64,
) -> Option<&'static str> {
    match source {
        BiomeSourceModel::Fixed { biome } => Some(*biome),
        BiomeSourceModel::Checkerboard { biomes, scale } => {
            let bit_shift = scale + 2;
            let index =
                ((quart_x >> bit_shift) + (quart_z >> bit_shift)).rem_euclid(biomes.len() as i32);
            Some(biomes[index as usize])
        }
        BiomeSourceModel::MultiNoisePreset {
            preset: "minecraft:nether",
        } => select_climate_biome(NETHER_BIOME_PARAMETERS, climate),
        BiomeSourceModel::MultiNoisePreset {
            preset: "minecraft:overworld",
        } => select_climate_biome(OVERWORLD_BIOME_PARAMETERS, climate),
        BiomeSourceModel::MultiNoisePreset { .. } => None,
        BiomeSourceModel::TheEnd => Some(select_end_biome(quart_x, quart_y, quart_z, end_erosion)),
    }
}

pub fn select_end_biome(
    quart_x: i32,
    quart_y: i32,
    quart_z: i32,
    erosion_value: f64,
) -> &'static str {
    let block_x = quart_x * 4;
    let block_z = quart_z * 4;
    let chunk_x = block_x.div_euclid(16);
    let chunk_z = block_z.div_euclid(16);
    if i64::from(chunk_x) * i64::from(chunk_x) + i64::from(chunk_z) * i64::from(chunk_z) <= 4096 {
        return "minecraft:the_end";
    }
    let _weird_block_x = (chunk_x * 2 + 1) * 8;
    let _weird_block_z = (chunk_z * 2 + 1) * 8;
    let _block_y = quart_y * 4;
    if erosion_value > 0.25 {
        "minecraft:end_highlands"
    } else if erosion_value >= -0.0625 {
        "minecraft:end_midlands"
    } else if erosion_value < -0.21875 {
        "minecraft:small_end_islands"
    } else {
        "minecraft:end_barrens"
    }
}

pub fn builtin_biome(id: &str) -> Option<&'static BiomeKey> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    BUILTIN_BIOMES.iter().find(|biome| {
        biome
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|biome_name| biome_name == name)
    })
}

const fn square(value: i64) -> i64 {
    value * value
}

#[cfg(test)]
mod tests {
    use super::{
        biome_source_codec, biome_source_from_stem_id, builtin_biome, checkerboard_biome_source,
        climate_node_distance, climate_point, climate_sampler_sample, climate_target,
        multi_noise_parameter_list_preset, quantize_coord, quart_to_block,
        select_biome_from_source, select_climate_biome, select_end_biome, unquantize_coord,
        BiomeSourceModel, ClimateBiomeEntry, ClimateParameter, ClimateParameterList,
        ClimateSamplerInput, BUILTIN_BIOMES, NETHER_BIOME_PARAMETERS, OVERWORLD_BIOME_PARAMETERS,
    };

    #[test]
    fn builtin_biome_registry_keys_match_26_1_2_biomes_order() {
        assert_eq!(BUILTIN_BIOMES.len(), 65);
        assert_eq!(BUILTIN_BIOMES.first().unwrap().id, "minecraft:the_void");
        assert_eq!(BUILTIN_BIOMES[1].id, "minecraft:plains");
        assert_eq!(BUILTIN_BIOMES[12].id, "minecraft:pale_garden");
        assert_eq!(BUILTIN_BIOMES[55].id, "minecraft:nether_wastes");
        assert_eq!(BUILTIN_BIOMES.last().unwrap().id, "minecraft:end_barrens");
        assert_eq!(builtin_biome("plains").unwrap().id, "minecraft:plains");
        assert!(builtin_biome("missing").is_none());
    }

    #[test]
    fn climate_quantization_and_parameter_distance_match_vanilla_rules() {
        assert_eq!(quantize_coord(0.375), 3750);
        assert_eq!(quantize_coord(-0.5), -5000);
        assert_eq!(unquantize_coord(3750), 0.375);

        let point = climate_point(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.175);
        assert_eq!(point.offset, 1750);
        assert_eq!(point.temperature.distance(quantize_coord(0.4)), 4000);
        assert_eq!(point.temperature.distance(quantize_coord(0.0)), 0);
    }

    #[test]
    fn climate_sampler_sample_uses_quart_to_block_context_before_quantization() {
        assert_eq!(quart_to_block(7), 28);
        assert_eq!(quart_to_block(-3), -12);

        let sample = climate_sampler_sample(
            7,
            -3,
            11,
            ClimateSamplerInput {
                temperature: 0.25,
                humidity: -0.125,
                continentalness: -0.5,
                erosion: 0.75,
                depth: 0.0,
                weirdness: 0.375,
            },
        );
        assert_eq!(sample.block_x, 28);
        assert_eq!(sample.block_y, -12);
        assert_eq!(sample.block_z, 44);
        assert_eq!(sample.target.temperature, 2500);
        assert_eq!(sample.target.humidity, -1250);
        assert_eq!(sample.target.continentalness, -5000);
        assert_eq!(sample.target.erosion, 7500);
        assert_eq!(sample.target.depth, 0);
        assert_eq!(sample.target.weirdness, 3750);
    }

    #[test]
    fn nether_multi_noise_parameters_select_nearest_biome() {
        assert_eq!(NETHER_BIOME_PARAMETERS.len(), 5);
        assert_eq!(
            select_climate_biome(
                NETHER_BIOME_PARAMETERS,
                climate_target(0.4, 0.0, 0.0, 0.0, 0.0, 0.0)
            ),
            Some("minecraft:crimson_forest")
        );
        assert_eq!(
            select_climate_biome(
                NETHER_BIOME_PARAMETERS,
                climate_target(0.0, 0.5, 0.0, 0.0, 0.0, 0.0)
            ),
            Some("minecraft:warped_forest")
        );
        assert_eq!(
            select_climate_biome(
                NETHER_BIOME_PARAMETERS,
                climate_target(-0.5, 0.0, 0.0, 0.0, 0.0, 0.0)
            ),
            Some("minecraft:basalt_deltas")
        );
    }

    #[test]
    fn multi_noise_parameter_list_presets_match_vanilla_known_presets() {
        let nether = multi_noise_parameter_list_preset("nether").unwrap();
        assert_eq!(nether.id, "minecraft:nether");
        assert_eq!(nether.parameters, NETHER_BIOME_PARAMETERS);
        assert_eq!(
            nether.used_biomes,
            &[
                "minecraft:nether_wastes",
                "minecraft:soul_sand_valley",
                "minecraft:crimson_forest",
                "minecraft:warped_forest",
                "minecraft:basalt_deltas"
            ]
        );

        let overworld = multi_noise_parameter_list_preset("minecraft:overworld").unwrap();
        assert_eq!(overworld.id, "minecraft:overworld");
        assert_eq!(overworld.parameters, OVERWORLD_BIOME_PARAMETERS);
        assert!(overworld.used_biomes.contains(&"minecraft:mushroom_fields"));
        assert!(overworld.used_biomes.contains(&"minecraft:plains"));
        assert!(overworld.used_biomes.contains(&"minecraft:deep_dark"));
        assert!(multi_noise_parameter_list_preset("missing").is_none());
    }

    #[test]
    fn overworld_multi_noise_representative_parameters_select_nearest_biome() {
        assert_eq!(OVERWORLD_BIOME_PARAMETERS.len(), 14);
        assert_eq!(
            select_climate_biome(
                OVERWORLD_BIOME_PARAMETERS,
                climate_target(0.0, 0.0, -1.1, 0.0, 0.0, 0.0)
            ),
            Some("minecraft:mushroom_fields")
        );
        assert_eq!(
            select_climate_biome(
                OVERWORLD_BIOME_PARAMETERS,
                climate_target(0.62, -0.2, 0.2, 0.0, 0.0, 0.0)
            ),
            Some("minecraft:desert")
        );
        assert_eq!(
            select_climate_biome(
                OVERWORLD_BIOME_PARAMETERS,
                climate_target(0.0, 0.0, 0.1, 0.0, 0.0, 0.0)
            ),
            Some("minecraft:forest")
        );

        let overworld = BiomeSourceModel::MultiNoisePreset {
            preset: "minecraft:overworld",
        };
        assert_eq!(
            select_biome_from_source(
                &overworld,
                0,
                0,
                0,
                climate_target(-0.3, -0.5, 0.0, 0.0, 0.0, 0.0),
                0.0
            ),
            Some("minecraft:plains")
        );
    }

    #[test]
    fn climate_selection_uses_first_entry_when_fitness_ties() {
        let entries = [
            ClimateBiomeEntry {
                parameters: climate_point(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
                biome: "minecraft:plains",
            },
            ClimateBiomeEntry {
                parameters: climate_point(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
                biome: "minecraft:forest",
            },
        ];
        assert_eq!(
            select_climate_biome(&entries, climate_target(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)),
            Some("minecraft:plains")
        );
    }

    #[test]
    fn climate_parameter_list_index_matches_bruteforce_search_contract() {
        let list = ClimateParameterList::new(vec![
            ClimateBiomeEntry {
                parameters: climate_point(-0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.175),
                biome: "minecraft:basalt_deltas",
            },
            ClimateBiomeEntry {
                parameters: climate_point(0.4, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
                biome: "minecraft:crimson_forest",
            },
            ClimateBiomeEntry {
                parameters: climate_point(0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.375),
                biome: "minecraft:warped_forest",
            },
        ])
        .unwrap();
        let target = climate_target(0.35, 0.02, 0.0, 0.0, 0.0, 0.0);
        assert_eq!(
            list.find_value_bruteforce(target),
            "minecraft:crimson_forest"
        );
        assert_eq!(list.find_value_index(target), "minecraft:crimson_forest");

        let nodes = list.rtree_nodes();
        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes[0].parameter_space.len(), 7);
        assert_eq!(
            climate_node_distance(nodes[0].parameter_space, target.parameter_array()),
            nodes[0]
                .parameter_space
                .into_iter()
                .zip(target.parameter_array())
                .map(|(parameter, target)| {
                    let distance = parameter.distance(target);
                    distance * distance
                })
                .sum::<i64>()
        );
        assert_eq!(
            ClimateParameter { min: -10, max: -2 }
                .distance_parameter(ClimateParameter { min: 3, max: 7 }),
            5
        );
        assert_eq!(
            ClimateParameter { min: -10, max: -2 }
                .span_parameter(Some(ClimateParameter { min: 3, max: 7 })),
            ClimateParameter { min: -10, max: 7 }
        );
        assert_eq!(
            ClimateParameterList::new(Vec::new()).unwrap_err(),
            "Need at least one value to build the search tree.".to_string()
        );
    }

    #[test]
    fn biome_source_codecs_match_vanilla_registry_order() {
        assert_eq!(biome_source_codec("fixed"), Some("minecraft:fixed"));
        assert_eq!(
            biome_source_codec("minecraft:multi_noise"),
            Some("minecraft:multi_noise")
        );
        assert_eq!(
            biome_source_codec("checkerboard"),
            Some("minecraft:checkerboard")
        );
        assert_eq!(biome_source_codec("the_end"), Some("minecraft:the_end"));
        assert_eq!(biome_source_codec("custom"), None);

        assert_eq!(
            biome_source_from_stem_id("minecraft:plains"),
            Some(BiomeSourceModel::Fixed {
                biome: "minecraft:plains"
            })
        );
        assert_eq!(
            biome_source_from_stem_id("multi_noise/nether"),
            Some(BiomeSourceModel::MultiNoisePreset {
                preset: "minecraft:nether"
            })
        );
        assert_eq!(
            biome_source_from_stem_id("the_end"),
            Some(BiomeSourceModel::TheEnd)
        );
    }

    #[test]
    fn fixed_checkerboard_nether_and_end_sources_select_like_vanilla() {
        let fixed = BiomeSourceModel::Fixed {
            biome: "minecraft:plains",
        };
        assert_eq!(
            select_biome_from_source(
                &fixed,
                120,
                -10,
                -44,
                climate_target(0.4, 0.0, 0.0, 0.0, 0.0, 0.0),
                0.0
            ),
            Some("minecraft:plains")
        );

        let checkerboard = checkerboard_biome_source(
            vec!["minecraft:plains", "minecraft:desert", "minecraft:forest"],
            2,
        )
        .unwrap();
        assert_eq!(
            select_biome_from_source(
                &checkerboard,
                0,
                0,
                0,
                climate_target(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
                0.0
            ),
            Some("minecraft:plains")
        );
        assert_eq!(
            select_biome_from_source(
                &checkerboard,
                16,
                0,
                0,
                climate_target(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
                0.0
            ),
            Some("minecraft:desert")
        );
        assert_eq!(
            checkerboard_biome_source(vec!["minecraft:plains"], 63).unwrap_err(),
            "Checkerboard biome source scale must be in 0..=62".to_string()
        );

        let nether = BiomeSourceModel::MultiNoisePreset {
            preset: "minecraft:nether",
        };
        assert_eq!(
            select_biome_from_source(
                &nether,
                0,
                0,
                0,
                climate_target(0.0, 0.5, 0.0, 0.0, 0.0, 0.0),
                0.0
            ),
            Some("minecraft:warped_forest")
        );

        assert_eq!(select_end_biome(0, 64, 0, -1.0), "minecraft:the_end");
        assert_eq!(select_end_biome(300, 64, 0, 0.3), "minecraft:end_highlands");
        assert_eq!(select_end_biome(300, 64, 0, 0.0), "minecraft:end_midlands");
        assert_eq!(
            select_end_biome(300, 64, 0, -0.3),
            "minecraft:small_end_islands"
        );
        assert_eq!(select_end_biome(300, 64, 0, -0.1), "minecraft:end_barrens");
    }
}
