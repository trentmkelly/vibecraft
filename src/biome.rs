#![allow(dead_code)]
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BiomeKey {
    pub id: &'static str,
}

/// Carvers and per-step placed-feature lists for a biome.
///
/// Mirrors Java's `BiomeGenerationSettings`:
/// - `carvers` is the flat `HolderSet<ConfiguredWorldCarver>` codec list; we store resource IDs.
/// - `features` has exactly 11 entries, one per `GenerationStep.Decoration` ordinal
///   (RAW_GENERATION=0 … TOP_LAYER_MODIFICATION=10).
///
/// Source: decompiled-server-26.1.2/net/minecraft/world/level/biome/BiomeGenerationSettings.java
///         decompiled-server-26.1.2/net/minecraft/world/level/levelgen/GenerationStep.java
#[derive(Debug, Clone, PartialEq)]
pub struct BiomeGenerationSettings {
    pub carvers: Vec<String>,
    /// One `Vec<String>` per `GenerationStep.Decoration` ordinal, 0 = RAW_GENERATION,
    /// 10 = TOP_LAYER_MODIFICATION. Always exactly 11 elements; absent steps are empty vecs.
    pub features: [Vec<String>; 11],
}

impl BiomeGenerationSettings {
    /// Serialized names of `GenerationStep.Decoration`, indexed by ordinal.
    pub const DECORATION_STEPS: [&'static str; 11] = [
        "raw_generation",
        "lakes",
        "local_modifications",
        "underground_structures",
        "surface_structures",
        "strongholds",
        "underground_ores",
        "underground_decoration",
        "fluid_springs",
        "vegetal_decoration",
        "top_layer_modification",
    ];
}

/// Mirrors Java's `net.minecraft.world.entity.MobCategory`.
/// Serialization keys come from `MobCategory.getSerializedName()`.
/// Source: decompiled-server-26.1.2/net/minecraft/world/entity/MobCategory.java
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MobCategory {
    Monster,
    Creature,
    Ambient,
    Axolotls,
    UndergroundWaterCreature,
    WaterCreature,
    WaterAmbient,
    Misc,
}

impl MobCategory {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "monster" => Some(MobCategory::Monster),
            "creature" => Some(MobCategory::Creature),
            "ambient" => Some(MobCategory::Ambient),
            "axolotls" => Some(MobCategory::Axolotls),
            "underground_water_creature" => Some(MobCategory::UndergroundWaterCreature),
            "water_creature" => Some(MobCategory::WaterCreature),
            "water_ambient" => Some(MobCategory::WaterAmbient),
            "misc" => Some(MobCategory::Misc),
            _ => None,
        }
    }

    pub fn serialized_name(self) -> &'static str {
        match self {
            MobCategory::Monster => "monster",
            MobCategory::Creature => "creature",
            MobCategory::Ambient => "ambient",
            MobCategory::Axolotls => "axolotls",
            MobCategory::UndergroundWaterCreature => "underground_water_creature",
            MobCategory::WaterCreature => "water_creature",
            MobCategory::WaterAmbient => "water_ambient",
            MobCategory::Misc => "misc",
        }
    }
}

/// One entry inside a `MobCategory` spawner list.
///
/// Mirrors Java's `MobSpawnSettings.SpawnerData` + the surrounding `WeightedList` codec:
/// `WeightedList.codec(SpawnerData.CODEC)` flattens the weight field into each entry JSON, so
/// the on-disk format is `{type, weight, minCount, maxCount}`.
/// Source: decompiled-server-26.1.2/net/minecraft/world/level/biome/MobSpawnSettings.java
///         decompiled-server-26.1.2/net/minecraft/util/random/WeightedList.java
///         decompiled-server-26.1.2/net/minecraft/util/random/Weighted.java
#[derive(Debug, Clone, PartialEq)]
pub struct SpawnerData {
    pub entity_type: String,
    pub weight: i32,
    pub min_count: i32,
    pub max_count: i32,
}

/// Mob spawn cost for natural spawning capacity tracking.
/// Mirrors Java's `MobSpawnSettings.MobSpawnCost`.
#[derive(Debug, Clone, PartialEq)]
pub struct SpawnCost {
    pub energy_budget: f64,
    pub charge: f64,
}

/// Mob spawn settings for a biome.
///
/// Mirrors Java's `MobSpawnSettings`:
/// - `creature_spawn_probability`: `optionalFieldOf("creature_spawn_probability", 0.1F)`.
/// - `spawners`: keyed by `MobCategory` serialized name, value is a `WeightedList<SpawnerData>`.
/// - `spawn_costs`: keyed by entity type resource ID, value is `{energy_budget, charge}`.
///
/// Source: decompiled-server-26.1.2/net/minecraft/world/level/biome/MobSpawnSettings.java
#[derive(Debug, Clone, PartialEq)]
pub struct MobSpawnSettings {
    /// Probability [0, 1) that a creature spawn attempt happens each chunk tick.
    /// Java default: `DEFAULT_CREATURE_SPAWN_PROBABILITY = 0.1F`.
    pub creature_spawn_probability: f32,
    pub spawners: HashMap<MobCategory, Vec<SpawnerData>>,
    pub spawn_costs: HashMap<String, SpawnCost>,
}

/// Full data for a single biome loaded from the registry JSON.
///
/// Mirrors the top-level codec in Java's `Biome.DIRECT_CODEC`:
/// `ClimateSettings` fields (has_precipitation, temperature, downfall) +
/// `BiomeGenerationSettings` + `MobSpawnSettings`.
///
/// Source: decompiled-server-26.1.2/net/minecraft/world/level/biome/Biome.java
#[derive(Debug, Clone, PartialEq)]
pub struct BiomeData {
    pub id: String,
    pub has_precipitation: bool,
    pub temperature: f32,
    pub downfall: f32,
    pub generation_settings: BiomeGenerationSettings,
    pub mob_spawn_settings: MobSpawnSettings,
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

#[derive(Debug, Clone, Copy, PartialEq)]
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

/// A `ParameterList` backed by a Climate R-tree for efficient nearest-neighbor lookup.
/// Mirrors Java's `Climate.ParameterList` which constructs an `RTree` at creation time.
/// Source: decompiled-server-26.1.2/net/minecraft/world/level/biome/Climate.java
#[derive(Debug, Clone)]
pub struct ClimateParameterList {
    pub values: Vec<ClimateBiomeEntry>,
    index: ClimateRTree,
}

/// An R-tree node used for efficient nearest-parameter-point search.
/// Each SubTree stores the bounding box of all descendant parameter spaces, enabling
/// pruning: if the bounding box is farther than the current best, the entire subtree
/// is skipped. Mirrors Java's Climate.RTree.Node / SubTree / Leaf.
#[derive(Debug, Clone)]
pub(crate) enum ClimateRTree {
    Leaf {
        parameter_space: [ClimateParameter; 7],
        biome: &'static str,
    },
    SubTree {
        /// Bounding box of all descendant parameter spaces.
        parameter_space: [ClimateParameter; 7],
        children: Vec<ClimateRTree>,
    },
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

pub const NETHER_MULTI_NOISE_USED_BIOMES: &[&str] = &[
    "minecraft:nether_wastes",
    "minecraft:soul_sand_valley",
    "minecraft:crimson_forest",
    "minecraft:warped_forest",
    "minecraft:basalt_deltas",
];

// All biomes reachable via the overworld OverworldBiomeBuilder parameter list.
// Source: derived from OverworldBiomeBuilder.addBiomes() picker functions and lookup tables.
pub const OVERWORLD_MULTI_NOISE_USED_BIOMES: &[&str] = &[
    // Off-coast
    "minecraft:mushroom_fields",
    "minecraft:deep_frozen_ocean",
    "minecraft:deep_cold_ocean",
    "minecraft:deep_ocean",
    "minecraft:deep_lukewarm_ocean",
    "minecraft:warm_ocean",
    "minecraft:frozen_ocean",
    "minecraft:cold_ocean",
    "minecraft:ocean",
    "minecraft:lukewarm_ocean",
    // Middle biomes
    "minecraft:snowy_plains",
    "minecraft:snowy_taiga",
    "minecraft:taiga",
    "minecraft:plains",
    "minecraft:forest",
    "minecraft:old_growth_spruce_taiga",
    "minecraft:flower_forest",
    "minecraft:birch_forest",
    "minecraft:dark_forest",
    "minecraft:savanna",
    "minecraft:jungle",
    "minecraft:desert",
    // Middle biome variants
    "minecraft:ice_spikes",
    "minecraft:old_growth_pine_taiga",
    "minecraft:sunflower_plains",
    "minecraft:old_growth_birch_forest",
    "minecraft:sparse_jungle",
    "minecraft:bamboo_jungle",
    // Plateau biomes
    "minecraft:meadow",
    "minecraft:pale_garden",
    "minecraft:savanna_plateau",
    "minecraft:badlands",
    "minecraft:wooded_badlands",
    // Plateau biome variants
    "minecraft:cherry_grove",
    "minecraft:eroded_badlands",
    // Shattered / windswept
    "minecraft:windswept_gravelly_hills",
    "minecraft:windswept_hills",
    "minecraft:windswept_forest",
    "minecraft:windswept_savanna",
    // Peaks and slopes
    "minecraft:jagged_peaks",
    "minecraft:frozen_peaks",
    "minecraft:stony_peaks",
    "minecraft:snowy_slopes",
    "minecraft:grove",
    // Coasts
    "minecraft:stony_shore",
    "minecraft:snowy_beach",
    "minecraft:beach",
    // Wetlands and rivers
    "minecraft:swamp",
    "minecraft:mangrove_swamp",
    "minecraft:river",
    "minecraft:frozen_river",
    // Underground
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
        let index = ClimateRTree::build(&values);
        Ok(Self { values, index })
    }

    pub fn find_value_bruteforce(&self, target: ClimateTarget) -> &'static str {
        let Some(entry) = self
            .values
            .iter()
            .min_by_key(|entry| entry.parameters.fitness(target))
        else {
            unreachable!("ClimateParameterList::new rejects empty parameter lists");
        };
        entry.biome
    }

    /// Finds the nearest biome using the R-tree index.
    /// Mirrors Java's `ParameterList.findValueIndex()` → `RTree.search()`.
    pub fn find_value_index(&self, target: ClimateTarget) -> &'static str {
        self.index.find_nearest(target.parameter_array())
    }
}

mod climate_rtree;

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
            parameters: overworld_biome_parameters(),
            used_biomes: OVERWORLD_MULTI_NOISE_USED_BIOMES,
        }),
        _ => None,
    }
}

/// Parses a `MultiNoiseBiomeSourceParameterList` JSON file and resolves the preset.
///
/// Both vanilla JSON files (`overworld.json`, `nether.json`) contain only a preset
/// reference — e.g. `{"preset":"minecraft:overworld"}` — so "loading" them just means
/// extracting the preset name and delegating to [`multi_noise_parameter_list_preset`].
/// Mirrors Java's `MultiNoiseBiomeSourceParameterList.CODEC` preset branch.
pub fn parse_multi_noise_preset_json(
    json: &str,
) -> Option<MultiNoiseBiomeSourceParameterListPreset> {
    // Minimal JSON extraction: find the value of "preset" without pulling in a dep.
    // Input is guaranteed to be `{"preset":"minecraft:X"}` for vanilla data files.
    let trimmed = json.trim();
    let inner = trimmed
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))?;
    for part in inner.split(',') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix("\"preset\"") {
            let value = rest.trim().strip_prefix(':')?.trim().trim_matches('"');
            return multi_noise_parameter_list_preset(value);
        }
    }
    None
}

/// Parses a biome registry JSON file and returns a [`BiomeData`] with generation settings and
/// mob spawn settings.
///
/// Implements the `Biome.DIRECT_CODEC` deserialization contract:
/// - Top-level `has_precipitation`, `temperature`, `downfall` come from `ClimateSettings`.
/// - `carvers`: flat list of carver resource IDs; vanilla JSON may encode a single holder as a
///   string or a holder set as an array.
/// - `features`: array of arrays; exactly 11 steps (absent trailing steps become empty vecs).
/// - `spawners`: object keyed by `MobCategory` serialized name; each value is an array of
///   `{type, weight, minCount, maxCount}` entries matching `WeightedList<SpawnerData>`.
/// - `spawn_costs`: object keyed by entity type; each value is `{energy_budget, charge}`.
/// - `creature_spawn_probability`: optional float, defaults to `0.1` per
///   `MobSpawnSettings.DEFAULT_CREATURE_SPAWN_PROBABILITY`.
///
/// Source: decompiled-server-26.1.2/net/minecraft/world/level/biome/Biome.java
///         decompiled-server-26.1.2/net/minecraft/world/level/biome/BiomeGenerationSettings.java
///         decompiled-server-26.1.2/net/minecraft/world/level/biome/MobSpawnSettings.java
pub fn parse_biome_json(id: &str, json: &str) -> Result<BiomeData, String> {
    let root: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("biome JSON parse error: {e}"))?;
    let obj = root
        .as_object()
        .ok_or("biome JSON root must be an object")?;

    let has_precipitation = obj
        .get("has_precipitation")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let temperature = obj
        .get("temperature")
        .and_then(|v| v.as_f64())
        .ok_or("missing 'temperature' field")? as f32;

    let downfall = obj
        .get("downfall")
        .and_then(|v| v.as_f64())
        .ok_or("missing 'downfall' field")? as f32;

    Ok(BiomeData {
        id: id.to_string(),
        has_precipitation,
        temperature,
        downfall,
        generation_settings: parse_generation_settings(obj)?,
        mob_spawn_settings: parse_mob_spawn_settings(obj)?,
    })
}

fn parse_generation_settings(
    obj: &serde_json::Map<String, serde_json::Value>,
) -> Result<BiomeGenerationSettings, String> {
    let carvers = match obj.get("carvers") {
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|value| value.as_str().map(String::from))
            .collect(),
        Some(serde_json::Value::String(carver)) => vec![carver.clone()],
        Some(_) => return Err("'carvers' must be an array or string".to_string()),
        None => Vec::new(),
    };
    Ok(BiomeGenerationSettings {
        carvers,
        features: parse_generation_features(obj)?,
    })
}

fn parse_generation_features(
    obj: &serde_json::Map<String, serde_json::Value>,
) -> Result<[Vec<String>; 11], String> {
    let empty_arr = Vec::new();
    let raw_features = match obj.get("features") {
        Some(serde_json::Value::Array(arr)) => arr,
        Some(_) => return Err("'features' must be an array".to_string()),
        None => &empty_arr,
    };

    let mut features: [Vec<String>; 11] = Default::default();
    for (step_idx, step_val) in raw_features.iter().take(11).enumerate() {
        let entries = step_val
            .as_array()
            .ok_or_else(|| format!("features[{step_idx}] must be an array"))?;
        features[step_idx] = entries
            .iter()
            .filter_map(|entry| entry.as_str().map(String::from))
            .collect();
    }
    Ok(features)
}

fn parse_mob_spawn_settings(
    obj: &serde_json::Map<String, serde_json::Value>,
) -> Result<MobSpawnSettings, String> {
    Ok(MobSpawnSettings {
        creature_spawn_probability: obj
            .get("creature_spawn_probability")
            .and_then(|value| value.as_f64())
            .unwrap_or(0.1) as f32,
        spawners: parse_spawners(obj)?,
        spawn_costs: parse_spawn_costs(obj)?,
    })
}

fn parse_spawners(
    obj: &serde_json::Map<String, serde_json::Value>,
) -> Result<HashMap<MobCategory, Vec<SpawnerData>>, String> {
    let mut spawners = HashMap::new();
    let Some(spawners_obj) = obj.get("spawners").and_then(|value| value.as_object()) else {
        return Ok(spawners);
    };
    for (cat_name, cat_val) in spawners_obj {
        let category = MobCategory::from_str(cat_name)
            .ok_or_else(|| format!("unknown MobCategory '{cat_name}'"))?;
        let entries = cat_val
            .as_array()
            .ok_or_else(|| format!("spawners['{cat_name}'] must be an array"))?;
        let data_list = entries
            .iter()
            .map(|entry| parse_spawner_entry(cat_name, entry))
            .collect::<Result<Vec<_>, _>>()?;
        spawners.insert(category, data_list);
    }
    Ok(spawners)
}

fn parse_spawner_entry(
    category_name: &str,
    entry: &serde_json::Value,
) -> Result<SpawnerData, String> {
    let object = entry
        .as_object()
        .ok_or_else(|| format!("spawners['{category_name}'] entries must be objects"))?;
    Ok(SpawnerData {
        entity_type: object
            .get("type")
            .and_then(|value| value.as_str())
            .ok_or("spawner entry missing 'type'")?
            .to_string(),
        weight: object
            .get("weight")
            .and_then(|value| value.as_i64())
            .ok_or("spawner entry missing 'weight'")? as i32,
        min_count: object
            .get("minCount")
            .and_then(|value| value.as_i64())
            .ok_or("spawner entry missing 'minCount'")? as i32,
        max_count: object
            .get("maxCount")
            .and_then(|value| value.as_i64())
            .ok_or("spawner entry missing 'maxCount'")? as i32,
    })
}

fn parse_spawn_costs(
    obj: &serde_json::Map<String, serde_json::Value>,
) -> Result<HashMap<String, SpawnCost>, String> {
    let mut spawn_costs = HashMap::new();
    let Some(costs_obj) = obj.get("spawn_costs").and_then(|value| value.as_object()) else {
        return Ok(spawn_costs);
    };
    for (entity_type, cost_val) in costs_obj {
        let cost = cost_val
            .as_object()
            .ok_or_else(|| format!("spawn_costs['{entity_type}'] must be an object"))?;
        spawn_costs.insert(
            entity_type.clone(),
            SpawnCost {
                energy_budget: cost
                    .get("energy_budget")
                    .and_then(|value| value.as_f64())
                    .ok_or("spawn cost missing 'energy_budget'")?,
                charge: cost
                    .get("charge")
                    .and_then(|value| value.as_f64())
                    .ok_or("spawn cost missing 'charge'")?,
            },
        );
    }
    Ok(spawn_costs)
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
        } => Some(overworld_biome_parameter_list().find_value_index(climate)),
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

// Lazily initialised overworld parameter list.
// Mirrors Java's `new OverworldBiomeBuilder().addBiomes(...)` call inside
// MultiNoiseBiomeSourceParameterList.
static OVERWORLD_BIOME_PARAMETERS_CACHE: OnceLock<Vec<ClimateBiomeEntry>> = OnceLock::new();
static OVERWORLD_BIOME_PARAMETER_LIST_CACHE: OnceLock<ClimateParameterList> = OnceLock::new();

pub fn overworld_biome_parameters() -> &'static [ClimateBiomeEntry] {
    OVERWORLD_BIOME_PARAMETERS_CACHE
        .get_or_init(|| overworld_builder::OverworldBiomeBuilder::new().build())
}

fn overworld_biome_parameter_list() -> &'static ClimateParameterList {
    OVERWORLD_BIOME_PARAMETER_LIST_CACHE.get_or_init(|| {
        match ClimateParameterList::new(overworld_biome_parameters().to_vec()) {
            Ok(list) => list,
            Err(_) => unreachable!("overworld biome builder always emits parameters"),
        }
    })
}

mod overworld_builder;

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests;
