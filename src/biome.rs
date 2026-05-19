#![allow(dead_code)]
use std::collections::HashMap;
use std::sync::OnceLock;

// Lookup tables mirroring OverworldBiomeBuilder's instance fields in Java.
// First index = temperature tier (0=coldest, 4=hottest).
// Second index = humidity tier (0=driest, 4=wettest).
// Source: decompiled-server-26.1.2/net/minecraft/world/level/biome/OverworldBiomeBuilder.java

const OCEANS: [[&str; 5]; 2] = [
    // Deep oceans (deepOceanContinentalness range)
    [
        "minecraft:deep_frozen_ocean",
        "minecraft:deep_cold_ocean",
        "minecraft:deep_ocean",
        "minecraft:deep_lukewarm_ocean",
        "minecraft:warm_ocean",
    ],
    // Shallow oceans (oceanContinentalness range)
    [
        "minecraft:frozen_ocean",
        "minecraft:cold_ocean",
        "minecraft:ocean",
        "minecraft:lukewarm_ocean",
        "minecraft:warm_ocean",
    ],
];

const MIDDLE_BIOMES: [[&str; 5]; 5] = [
    [
        "minecraft:snowy_plains",
        "minecraft:snowy_plains",
        "minecraft:snowy_plains",
        "minecraft:snowy_taiga",
        "minecraft:taiga",
    ],
    [
        "minecraft:plains",
        "minecraft:plains",
        "minecraft:forest",
        "minecraft:taiga",
        "minecraft:old_growth_spruce_taiga",
    ],
    [
        "minecraft:flower_forest",
        "minecraft:plains",
        "minecraft:forest",
        "minecraft:birch_forest",
        "minecraft:dark_forest",
    ],
    [
        "minecraft:savanna",
        "minecraft:savanna",
        "minecraft:forest",
        "minecraft:jungle",
        "minecraft:jungle",
    ],
    [
        "minecraft:desert",
        "minecraft:desert",
        "minecraft:desert",
        "minecraft:desert",
        "minecraft:desert",
    ],
];

const MIDDLE_BIOMES_VARIANT: [[Option<&str>; 5]; 5] = [
    [
        Some("minecraft:ice_spikes"),
        None,
        Some("minecraft:snowy_taiga"),
        None,
        None,
    ],
    [
        None,
        None,
        None,
        None,
        Some("minecraft:old_growth_pine_taiga"),
    ],
    [
        Some("minecraft:sunflower_plains"),
        None,
        None,
        Some("minecraft:old_growth_birch_forest"),
        None,
    ],
    [
        None,
        None,
        Some("minecraft:plains"),
        Some("minecraft:sparse_jungle"),
        Some("minecraft:bamboo_jungle"),
    ],
    [None, None, None, None, None],
];

const PLATEAU_BIOMES: [[&str; 5]; 5] = [
    [
        "minecraft:snowy_plains",
        "minecraft:snowy_plains",
        "minecraft:snowy_plains",
        "minecraft:snowy_taiga",
        "minecraft:snowy_taiga",
    ],
    [
        "minecraft:meadow",
        "minecraft:meadow",
        "minecraft:forest",
        "minecraft:taiga",
        "minecraft:old_growth_spruce_taiga",
    ],
    [
        "minecraft:meadow",
        "minecraft:meadow",
        "minecraft:meadow",
        "minecraft:meadow",
        "minecraft:pale_garden",
    ],
    [
        "minecraft:savanna_plateau",
        "minecraft:savanna_plateau",
        "minecraft:forest",
        "minecraft:forest",
        "minecraft:jungle",
    ],
    [
        "minecraft:badlands",
        "minecraft:badlands",
        "minecraft:badlands",
        "minecraft:wooded_badlands",
        "minecraft:wooded_badlands",
    ],
];

const PLATEAU_BIOMES_VARIANT: [[Option<&str>; 5]; 5] = [
    [Some("minecraft:ice_spikes"), None, None, None, None],
    [
        Some("minecraft:cherry_grove"),
        None,
        Some("minecraft:meadow"),
        Some("minecraft:meadow"),
        Some("minecraft:old_growth_pine_taiga"),
    ],
    [
        Some("minecraft:cherry_grove"),
        Some("minecraft:cherry_grove"),
        Some("minecraft:forest"),
        Some("minecraft:birch_forest"),
        None,
    ],
    [None, None, None, None, None],
    [
        Some("minecraft:eroded_badlands"),
        Some("minecraft:eroded_badlands"),
        None,
        None,
        None,
    ],
];

const SHATTERED_BIOMES: [[Option<&str>; 5]; 5] = [
    [
        Some("minecraft:windswept_gravelly_hills"),
        Some("minecraft:windswept_gravelly_hills"),
        Some("minecraft:windswept_hills"),
        Some("minecraft:windswept_forest"),
        Some("minecraft:windswept_forest"),
    ],
    [
        Some("minecraft:windswept_gravelly_hills"),
        Some("minecraft:windswept_gravelly_hills"),
        Some("minecraft:windswept_hills"),
        Some("minecraft:windswept_forest"),
        Some("minecraft:windswept_forest"),
    ],
    [
        Some("minecraft:windswept_hills"),
        Some("minecraft:windswept_hills"),
        Some("minecraft:windswept_hills"),
        Some("minecraft:windswept_forest"),
        Some("minecraft:windswept_forest"),
    ],
    [None, None, None, None, None],
    [None, None, None, None, None],
];

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
        self.values
            .iter()
            .min_by_key(|entry| entry.parameters.fitness(target))
            .expect("parameter list is non-empty")
            .biome
    }

    /// Finds the nearest biome using the R-tree index.
    /// Mirrors Java's `ParameterList.findValueIndex()` → `RTree.search()`.
    pub fn find_value_index(&self, target: ClimateTarget) -> &'static str {
        self.index.find_nearest(target.parameter_array())
    }
}

impl ClimateRTree {
    /// Builds an R-tree from a parameter list.
    /// Mirrors Java's `Climate.RTree.create()`.
    pub fn build(entries: &[ClimateBiomeEntry]) -> Self {
        assert!(
            !entries.is_empty(),
            "Need at least one value to build the search tree."
        );
        let leaves: Vec<Self> = entries
            .iter()
            .map(|e| Self::Leaf {
                parameter_space: e.parameters.parameter_space(),
                biome: e.biome,
            })
            .collect();
        Self::build_internal(leaves)
    }

    fn parameter_space(&self) -> &[ClimateParameter; 7] {
        match self {
            Self::Leaf {
                parameter_space, ..
            }
            | Self::SubTree {
                parameter_space, ..
            } => parameter_space,
        }
    }

    /// Sum of squared per-dimension distances. For SubTree nodes this is a lower-bound
    /// used for pruning; for Leaf nodes it is the exact distance.
    /// Mirrors Java's `Node.distance(long[] target)`.
    fn node_distance(&self, target: &[i64; 7]) -> i64 {
        self.parameter_space()
            .iter()
            .zip(target.iter())
            .map(|(p, &t)| square(p.distance(t)))
            .sum()
    }

    fn bounding_box_of(
        mut iter: impl Iterator<Item = [ClimateParameter; 7]>,
    ) -> [ClimateParameter; 7] {
        let first = iter.next().expect("non-empty iterator");
        iter.fold(first, |mut acc, ps| {
            for i in 0..7 {
                acc[i] = acc[i].span_parameter(Some(ps[i]));
            }
            acc
        })
    }

    fn bounding_box_cost(ps: &[ClimateParameter; 7]) -> i64 {
        ps.iter().map(|p| (p.max - p.min).abs()).sum()
    }

    /// Recursive tree builder. Mirrors Java's `RTree.build(dimensions, children)`.
    ///
    /// - Size 1: return the single node directly.
    /// - Size ≤ 6: sort by |center| magnitude, wrap in a SubTree.
    /// - Size > 6: try each of 7 dimensions; pick the one minimising total
    ///   bounding-box cost across buckets of size 6^floor(log(n−0.01)/log(6)).
    ///   Sort winning buckets by |center| of that dimension, recurse into each.
    fn build_internal(nodes: Vec<Self>) -> Self {
        const MAX_CHILDREN: usize = 6;
        const DIMS: usize = 7;

        if nodes.len() == 1 {
            return nodes.into_iter().next().unwrap();
        }

        if nodes.len() <= MAX_CHILDREN {
            let mut nodes = nodes;
            nodes.sort_by_key(|n| {
                n.parameter_space()
                    .iter()
                    .map(|p| ((p.min + p.max) / 2).abs())
                    .sum::<i64>()
            });
            let ps = Self::bounding_box_of(nodes.iter().map(|n| *n.parameter_space()));
            return Self::SubTree {
                parameter_space: ps,
                children: nodes,
            };
        }

        let n = nodes.len();
        // Java: (int)Math.pow(6.0, Math.floor(Math.log(n - 0.01) / Math.log(6.0)))
        let expected: usize = {
            let log6 = 6.0_f64.ln();
            let exp = ((n as f64 - 0.01).ln() / log6).floor() as u32;
            6usize.pow(exp).max(1)
        };

        let center = |nodes: &[Self], i: usize, d: usize| -> i64 {
            let p = &nodes[i].parameter_space()[d];
            (p.min + p.max) / 2
        };

        let mut indices: Vec<usize> = (0..n).collect();
        let mut min_cost = i64::MAX;
        let mut best_dim = 0usize;

        for d in 0..DIMS {
            // Lexicographic sort: primary = dim d, tiebreak by (d+1)%7 … (d+6)%7.
            // Matches Java's `sort(children, dimensions, d, false)`.
            indices.sort_by(|&a, &b| {
                for k in 0..DIMS {
                    let dim = (d + k) % DIMS;
                    match center(&nodes, a, dim).cmp(&center(&nodes, b, dim)) {
                        std::cmp::Ordering::Equal => {}
                        ord => return ord,
                    }
                }
                std::cmp::Ordering::Equal
            });
            let cost: i64 = indices
                .chunks(expected)
                .map(|bucket| {
                    let ps =
                        Self::bounding_box_of(bucket.iter().map(|&i| *nodes[i].parameter_space()));
                    Self::bounding_box_cost(&ps)
                })
                .sum();
            if cost < min_cost {
                min_cost = cost;
                best_dim = d;
            }
        }

        // Re-sort by the winning dimension (matches Java's re-sort before grouping).
        indices.sort_by(|&a, &b| {
            for k in 0..DIMS {
                let dim = (best_dim + k) % DIMS;
                match center(&nodes, a, dim).cmp(&center(&nodes, b, dim)) {
                    std::cmp::Ordering::Equal => {}
                    ord => return ord,
                }
            }
            std::cmp::Ordering::Equal
        });

        // Group into buckets of size `expected`.
        let mut bucket_groups: Vec<Vec<usize>> = {
            let mut groups: Vec<Vec<usize>> = Vec::new();
            let mut current: Vec<usize> = Vec::with_capacity(expected);
            for &i in &indices {
                current.push(i);
                if current.len() >= expected {
                    groups.push(std::mem::replace(
                        &mut current,
                        Vec::with_capacity(expected),
                    ));
                }
            }
            if !current.is_empty() {
                groups.push(current);
            }
            groups
        };

        // Sort bucket groups by |center| of best_dim (matches Java's `sort(minBuckets, …, true)`).
        bucket_groups.sort_by_key(|bucket| {
            let (lo, hi) = bucket.iter().fold((i64::MAX, i64::MIN), |(lo, hi), &i| {
                let p = &nodes[i].parameter_space()[best_dim];
                (lo.min(p.min), hi.max(p.max))
            });
            ((lo + hi) / 2).abs()
        });

        // Consume nodes into buckets and recursively build each.
        let mut slots: Vec<Option<Self>> = nodes.into_iter().map(Some).collect();
        let children: Vec<Self> = bucket_groups
            .into_iter()
            .map(|bucket| {
                let bucket_nodes: Vec<Self> = bucket
                    .into_iter()
                    .map(|i| slots[i].take().unwrap())
                    .collect();
                Self::build_internal(bucket_nodes)
            })
            .collect();

        let ps = Self::bounding_box_of(children.iter().map(|c| *c.parameter_space()));
        Self::SubTree {
            parameter_space: ps,
            children,
        }
    }

    /// Entry point for nearest-neighbour search. Mirrors Java's `RTree.search()`.
    pub fn find_nearest(&self, target: [i64; 7]) -> &'static str {
        match self {
            Self::Leaf { biome, .. } => biome,
            Self::SubTree { .. } => {
                let mut best_dist = i64::MAX;
                let mut best_biome = "";
                self.search_subtree(&target, &mut best_dist, &mut best_biome);
                best_biome
            }
        }
    }

    /// Recursive subtree search with bounding-box pruning.
    /// Mirrors Java's `SubTree.search()`.
    fn search_subtree(
        &self,
        target: &[i64; 7],
        best_dist: &mut i64,
        best_biome: &mut &'static str,
    ) {
        if let Self::SubTree { children, .. } = self {
            for child in children {
                let bound = child.node_distance(target);
                if bound < *best_dist {
                    match child {
                        Self::Leaf { biome, .. } => {
                            *best_dist = bound;
                            *best_biome = biome;
                        }
                        Self::SubTree { .. } => {
                            child.search_subtree(target, best_dist, best_biome);
                        }
                    }
                }
            }
        }
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
/// - `carvers`: flat list of carver resource IDs.
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

    // --- BiomeGenerationSettings ---

    let carvers: Vec<String> = match obj.get("carvers") {
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        Some(_) => return Err("'carvers' must be an array".to_string()),
        None => Vec::new(),
    };

    let empty_arr: Vec<serde_json::Value> = Vec::new();
    let raw_features: &Vec<serde_json::Value> = match obj.get("features") {
        Some(serde_json::Value::Array(arr)) => arr,
        Some(_) => return Err("'features' must be an array".to_string()),
        None => &empty_arr,
    };

    let mut features: [Vec<String>; 11] = Default::default();
    for (step_idx, step_val) in raw_features.iter().enumerate() {
        if step_idx >= 11 {
            break;
        }
        match step_val.as_array() {
            Some(entries) => {
                features[step_idx] = entries
                    .iter()
                    .filter_map(|e| e.as_str().map(String::from))
                    .collect();
            }
            None => return Err(format!("features[{step_idx}] must be an array")),
        }
    }

    let generation_settings = BiomeGenerationSettings { carvers, features };

    // --- MobSpawnSettings ---

    let creature_spawn_probability = obj
        .get("creature_spawn_probability")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.1) as f32;

    let mut spawners: HashMap<MobCategory, Vec<SpawnerData>> = HashMap::new();
    if let Some(spawners_obj) = obj.get("spawners").and_then(|v| v.as_object()) {
        for (cat_name, cat_val) in spawners_obj {
            let category = MobCategory::from_str(cat_name)
                .ok_or_else(|| format!("unknown MobCategory '{cat_name}'"))?;
            let entries = cat_val
                .as_array()
                .ok_or_else(|| format!("spawners['{cat_name}'] must be an array"))?;
            let mut data_list: Vec<SpawnerData> = Vec::with_capacity(entries.len());
            for entry in entries {
                let e = entry
                    .as_object()
                    .ok_or_else(|| format!("spawners['{cat_name}'] entries must be objects"))?;
                let entity_type = e
                    .get("type")
                    .and_then(|v| v.as_str())
                    .ok_or("spawner entry missing 'type'")?
                    .to_string();
                let weight = e
                    .get("weight")
                    .and_then(|v| v.as_i64())
                    .ok_or("spawner entry missing 'weight'")? as i32;
                let min_count = e
                    .get("minCount")
                    .and_then(|v| v.as_i64())
                    .ok_or("spawner entry missing 'minCount'")? as i32;
                let max_count = e
                    .get("maxCount")
                    .and_then(|v| v.as_i64())
                    .ok_or("spawner entry missing 'maxCount'")? as i32;
                data_list.push(SpawnerData {
                    entity_type,
                    weight,
                    min_count,
                    max_count,
                });
            }
            spawners.insert(category, data_list);
        }
    }

    let mut spawn_costs: HashMap<String, SpawnCost> = HashMap::new();
    if let Some(costs_obj) = obj.get("spawn_costs").and_then(|v| v.as_object()) {
        for (entity_type, cost_val) in costs_obj {
            let c = cost_val
                .as_object()
                .ok_or_else(|| format!("spawn_costs['{entity_type}'] must be an object"))?;
            let energy_budget = c
                .get("energy_budget")
                .and_then(|v| v.as_f64())
                .ok_or("spawn cost missing 'energy_budget'")?;
            let charge = c
                .get("charge")
                .and_then(|v| v.as_f64())
                .ok_or("spawn cost missing 'charge'")?;
            spawn_costs.insert(entity_type.clone(), SpawnCost { energy_budget, charge });
        }
    }

    let mob_spawn_settings = MobSpawnSettings {
        creature_spawn_probability,
        spawners,
        spawn_costs,
    };

    Ok(BiomeData {
        id: id.to_string(),
        has_precipitation,
        temperature,
        downfall,
        generation_settings,
        mob_spawn_settings,
    })
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
        } => select_climate_biome(overworld_biome_parameters(), climate),
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

pub fn overworld_biome_parameters() -> &'static [ClimateBiomeEntry] {
    OVERWORLD_BIOME_PARAMETERS_CACHE.get_or_init(|| OverworldBiomeBuilder::new().build())
}

// ---- OverworldBiomeBuilder ----
// Mirrors Java's net.minecraft.world.level.biome.OverworldBiomeBuilder.
// Invoked once via overworld_biome_parameters(); result cached in OVERWORLD_BIOME_PARAMETERS_CACHE.
struct OverworldBiomeBuilder {
    temperatures: [ClimateParameter; 5],
    humidities: [ClimateParameter; 5],
    erosions: [ClimateParameter; 7],
    full_range: ClimateParameter,
    frozen_range: ClimateParameter,
    unfrozen_range: ClimateParameter,
    mushroom_fields_cont: ClimateParameter,
    deep_ocean_cont: ClimateParameter,
    ocean_cont: ClimateParameter,
    coast_cont: ClimateParameter,
    inland_cont: ClimateParameter,
    near_inland_cont: ClimateParameter,
    mid_inland_cont: ClimateParameter,
    far_inland_cont: ClimateParameter,
}

impl OverworldBiomeBuilder {
    fn new() -> Self {
        let temperatures = [
            span(-1.0, -0.45),
            span(-0.45, -0.15),
            span(-0.15, 0.2),
            span(0.2, 0.55),
            span(0.55, 1.0),
        ];
        let frozen_range = temperatures[0];
        // Java: Climate.Parameter.span(temperatures[1], temperatures[4]) — min of [1], max of [4]
        let unfrozen_range = ClimateParameter {
            min: temperatures[1].min,
            max: temperatures[4].max,
        };
        let near_inland_cont = span(-0.11, 0.03);
        let mid_inland_cont = span(0.03, 0.3);
        let far_inland_cont = span(0.3, 1.0);
        Self {
            temperatures,
            humidities: [
                span(-1.0, -0.35),
                span(-0.35, -0.1),
                span(-0.1, 0.1),
                span(0.1, 0.3),
                span(0.3, 1.0),
            ],
            erosions: [
                span(-1.0, -0.78),
                span(-0.78, -0.375),
                span(-0.375, -0.2225),
                span(-0.2225, 0.05),
                span(0.05, 0.45),
                span(0.45, 0.55),
                span(0.55, 1.0),
            ],
            full_range: span(-1.0, 1.0),
            frozen_range,
            unfrozen_range,
            mushroom_fields_cont: span(-1.2, -1.05),
            deep_ocean_cont: span(-1.05, -0.455),
            ocean_cont: span(-0.455, -0.19),
            coast_cont: span(-0.19, -0.11),
            inland_cont: span(-0.11, 0.55),
            near_inland_cont,
            mid_inland_cont,
            far_inland_cont,
        }
    }

    fn build(self) -> Vec<ClimateBiomeEntry> {
        let mut entries = Vec::new();
        self.add_off_coast_biomes(&mut entries);
        self.add_inland_biomes(&mut entries);
        self.add_underground_biomes(&mut entries);
        entries
    }

    // Java Climate.Parameter.span(Parameter a, Parameter b): { min: a.min, max: b.max }
    fn p_span(a: ClimateParameter, b: ClimateParameter) -> ClimateParameter {
        ClimateParameter {
            min: a.min,
            max: b.max,
        }
    }

    // Java addSurfaceBiome emits two entries: depth=point(0.0) and depth=point(1.0)
    fn add_surface_biome(
        &self,
        entries: &mut Vec<ClimateBiomeEntry>,
        temp: ClimateParameter,
        hum: ClimateParameter,
        cont: ClimateParameter,
        eros: ClimateParameter,
        weird: ClimateParameter,
        offset: f32,
        biome: &'static str,
    ) {
        for depth_val in [0.0_f32, 1.0_f32] {
            entries.push(ClimateBiomeEntry {
                parameters: ClimateParameterPoint {
                    temperature: temp,
                    humidity: hum,
                    continentalness: cont,
                    erosion: eros,
                    depth: point(depth_val),
                    weirdness: weird,
                    offset: quantize_coord(offset),
                },
                biome,
            });
        }
    }

    // Java addUndergroundBiome emits one entry with depth=span(0.2, 0.9)
    fn add_underground_biome(
        &self,
        entries: &mut Vec<ClimateBiomeEntry>,
        temp: ClimateParameter,
        hum: ClimateParameter,
        cont: ClimateParameter,
        eros: ClimateParameter,
        weird: ClimateParameter,
        offset: f32,
        biome: &'static str,
    ) {
        entries.push(ClimateBiomeEntry {
            parameters: ClimateParameterPoint {
                temperature: temp,
                humidity: hum,
                continentalness: cont,
                erosion: eros,
                depth: span(0.2, 0.9),
                weirdness: weird,
                offset: quantize_coord(offset),
            },
            biome,
        });
    }

    // Java addBottomBiome emits one entry with depth=point(1.1)
    fn add_bottom_biome(
        &self,
        entries: &mut Vec<ClimateBiomeEntry>,
        temp: ClimateParameter,
        hum: ClimateParameter,
        cont: ClimateParameter,
        eros: ClimateParameter,
        weird: ClimateParameter,
        offset: f32,
        biome: &'static str,
    ) {
        entries.push(ClimateBiomeEntry {
            parameters: ClimateParameterPoint {
                temperature: temp,
                humidity: hum,
                continentalness: cont,
                erosion: eros,
                depth: point(1.1),
                weirdness: weird,
                offset: quantize_coord(offset),
            },
            biome,
        });
    }

    // -- Biome picker functions --

    fn pick_middle_biome(&self, ti: usize, hi: usize, weird: ClimateParameter) -> &'static str {
        if weird.max < 0 {
            MIDDLE_BIOMES[ti][hi]
        } else {
            MIDDLE_BIOMES_VARIANT[ti][hi].unwrap_or(MIDDLE_BIOMES[ti][hi])
        }
    }

    fn pick_middle_biome_or_badlands_if_hot(
        &self,
        ti: usize,
        hi: usize,
        weird: ClimateParameter,
    ) -> &'static str {
        if ti == 4 {
            self.pick_badlands_biome(hi, weird)
        } else {
            self.pick_middle_biome(ti, hi, weird)
        }
    }

    fn pick_middle_biome_or_badlands_if_hot_or_slope_if_cold(
        &self,
        ti: usize,
        hi: usize,
        weird: ClimateParameter,
    ) -> &'static str {
        if ti == 0 {
            self.pick_slope_biome(ti, hi, weird)
        } else {
            self.pick_middle_biome_or_badlands_if_hot(ti, hi, weird)
        }
    }

    fn maybe_pick_windswept_savanna_biome(
        &self,
        ti: usize,
        hi: usize,
        weird: ClimateParameter,
        fallback: &'static str,
    ) -> &'static str {
        if ti > 1 && hi < 4 && weird.max >= 0 {
            "minecraft:windswept_savanna"
        } else {
            fallback
        }
    }

    fn pick_shattered_coast_biome(
        &self,
        ti: usize,
        hi: usize,
        weird: ClimateParameter,
    ) -> &'static str {
        let base = if weird.max >= 0 {
            self.pick_middle_biome(ti, hi, weird)
        } else {
            self.pick_beach_biome(ti)
        };
        self.maybe_pick_windswept_savanna_biome(ti, hi, weird, base)
    }

    fn pick_beach_biome(&self, ti: usize) -> &'static str {
        match ti {
            0 => "minecraft:snowy_beach",
            4 => "minecraft:desert",
            _ => "minecraft:beach",
        }
    }

    fn pick_badlands_biome(&self, hi: usize, weird: ClimateParameter) -> &'static str {
        if hi < 2 {
            if weird.max < 0 {
                "minecraft:badlands"
            } else {
                "minecraft:eroded_badlands"
            }
        } else if hi < 3 {
            "minecraft:badlands"
        } else {
            "minecraft:wooded_badlands"
        }
    }

    fn pick_plateau_biome(&self, ti: usize, hi: usize, weird: ClimateParameter) -> &'static str {
        if weird.max >= 0 {
            if let Some(variant) = PLATEAU_BIOMES_VARIANT[ti][hi] {
                return variant;
            }
        }
        PLATEAU_BIOMES[ti][hi]
    }

    fn pick_peak_biome(&self, ti: usize, hi: usize, weird: ClimateParameter) -> &'static str {
        if ti <= 2 {
            if weird.max < 0 {
                "minecraft:jagged_peaks"
            } else {
                "minecraft:frozen_peaks"
            }
        } else if ti == 3 {
            "minecraft:stony_peaks"
        } else {
            self.pick_badlands_biome(hi, weird)
        }
    }

    fn pick_slope_biome(&self, ti: usize, hi: usize, weird: ClimateParameter) -> &'static str {
        if ti >= 3 {
            self.pick_plateau_biome(ti, hi, weird)
        } else if hi <= 1 {
            "minecraft:snowy_slopes"
        } else {
            "minecraft:grove"
        }
    }

    fn pick_shattered_biome(&self, ti: usize, hi: usize, weird: ClimateParameter) -> &'static str {
        SHATTERED_BIOMES[ti][hi].unwrap_or_else(|| self.pick_middle_biome(ti, hi, weird))
    }

    // -- Inland slice generators --

    fn add_peaks(&self, entries: &mut Vec<ClimateBiomeEntry>, weird: ClimateParameter) {
        for ti in 0..5_usize {
            let temp = self.temperatures[ti];
            for hi in 0..5_usize {
                let hum = self.humidities[hi];
                let middle = self.pick_middle_biome(ti, hi, weird);
                let middle_or_badlands = self.pick_middle_biome_or_badlands_if_hot(ti, hi, weird);
                let middle_or_badlands_or_slope =
                    self.pick_middle_biome_or_badlands_if_hot_or_slope_if_cold(ti, hi, weird);
                let plateau = self.pick_plateau_biome(ti, hi, weird);
                let shattered = self.pick_shattered_biome(ti, hi, weird);
                let shattered_or_windswept =
                    self.maybe_pick_windswept_savanna_biome(ti, hi, weird, shattered);
                let peak = self.pick_peak_biome(ti, hi, weird);

                let coast_far = Self::p_span(self.coast_cont, self.far_inland_cont);
                let coast_near = Self::p_span(self.coast_cont, self.near_inland_cont);
                let mid_far = Self::p_span(self.mid_inland_cont, self.far_inland_cont);
                let eros23 = Self::p_span(self.erosions[2], self.erosions[3]);

                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    coast_far,
                    self.erosions[0],
                    weird,
                    0.0,
                    peak,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    coast_near,
                    self.erosions[1],
                    weird,
                    0.0,
                    middle_or_badlands_or_slope,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    mid_far,
                    self.erosions[1],
                    weird,
                    0.0,
                    peak,
                );
                self.add_surface_biome(entries, temp, hum, coast_near, eros23, weird, 0.0, middle);
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    mid_far,
                    self.erosions[2],
                    weird,
                    0.0,
                    plateau,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.mid_inland_cont,
                    self.erosions[3],
                    weird,
                    0.0,
                    middle_or_badlands,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.far_inland_cont,
                    self.erosions[3],
                    weird,
                    0.0,
                    plateau,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    coast_far,
                    self.erosions[4],
                    weird,
                    0.0,
                    middle,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    coast_near,
                    self.erosions[5],
                    weird,
                    0.0,
                    shattered_or_windswept,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    mid_far,
                    self.erosions[5],
                    weird,
                    0.0,
                    shattered,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    coast_far,
                    self.erosions[6],
                    weird,
                    0.0,
                    middle,
                );
            }
        }
    }

    fn add_high_slice(&self, entries: &mut Vec<ClimateBiomeEntry>, weird: ClimateParameter) {
        for ti in 0..5_usize {
            let temp = self.temperatures[ti];
            for hi in 0..5_usize {
                let hum = self.humidities[hi];
                let middle = self.pick_middle_biome(ti, hi, weird);
                let middle_or_badlands = self.pick_middle_biome_or_badlands_if_hot(ti, hi, weird);
                let middle_or_badlands_or_slope =
                    self.pick_middle_biome_or_badlands_if_hot_or_slope_if_cold(ti, hi, weird);
                let plateau = self.pick_plateau_biome(ti, hi, weird);
                let shattered = self.pick_shattered_biome(ti, hi, weird);
                let middle_or_windswept =
                    self.maybe_pick_windswept_savanna_biome(ti, hi, weird, middle);
                let slope = self.pick_slope_biome(ti, hi, weird);
                let peak = self.pick_peak_biome(ti, hi, weird);

                let coast_near = Self::p_span(self.coast_cont, self.near_inland_cont);
                let mid_far = Self::p_span(self.mid_inland_cont, self.far_inland_cont);
                let coast_far = Self::p_span(self.coast_cont, self.far_inland_cont);
                let eros01 = Self::p_span(self.erosions[0], self.erosions[1]);
                let eros23 = Self::p_span(self.erosions[2], self.erosions[3]);

                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.coast_cont,
                    eros01,
                    weird,
                    0.0,
                    middle,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.near_inland_cont,
                    self.erosions[0],
                    weird,
                    0.0,
                    slope,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    mid_far,
                    self.erosions[0],
                    weird,
                    0.0,
                    peak,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.near_inland_cont,
                    self.erosions[1],
                    weird,
                    0.0,
                    middle_or_badlands_or_slope,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    mid_far,
                    self.erosions[1],
                    weird,
                    0.0,
                    slope,
                );
                self.add_surface_biome(entries, temp, hum, coast_near, eros23, weird, 0.0, middle);
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    mid_far,
                    self.erosions[2],
                    weird,
                    0.0,
                    plateau,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.mid_inland_cont,
                    self.erosions[3],
                    weird,
                    0.0,
                    middle_or_badlands,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.far_inland_cont,
                    self.erosions[3],
                    weird,
                    0.0,
                    plateau,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    coast_far,
                    self.erosions[4],
                    weird,
                    0.0,
                    middle,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    coast_near,
                    self.erosions[5],
                    weird,
                    0.0,
                    middle_or_windswept,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    mid_far,
                    self.erosions[5],
                    weird,
                    0.0,
                    shattered,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    coast_far,
                    self.erosions[6],
                    weird,
                    0.0,
                    middle,
                );
            }
        }
    }

    fn add_mid_slice(&self, entries: &mut Vec<ClimateBiomeEntry>, weird: ClimateParameter) {
        let near_far = Self::p_span(self.near_inland_cont, self.far_inland_cont);
        let coast_near = Self::p_span(self.coast_cont, self.near_inland_cont);
        let near_mid = Self::p_span(self.near_inland_cont, self.mid_inland_cont);
        let mid_far = Self::p_span(self.mid_inland_cont, self.far_inland_cont);
        let coast_far = Self::p_span(self.coast_cont, self.far_inland_cont);
        let eros02 = Self::p_span(self.erosions[0], self.erosions[2]);
        let temp12 = Self::p_span(self.temperatures[1], self.temperatures[2]);
        let temp34 = Self::p_span(self.temperatures[3], self.temperatures[4]);

        self.add_surface_biome(
            entries,
            self.full_range,
            self.full_range,
            self.coast_cont,
            eros02,
            weird,
            0.0,
            "minecraft:stony_shore",
        );
        self.add_surface_biome(
            entries,
            temp12,
            self.full_range,
            near_far,
            self.erosions[6],
            weird,
            0.0,
            "minecraft:swamp",
        );
        self.add_surface_biome(
            entries,
            temp34,
            self.full_range,
            near_far,
            self.erosions[6],
            weird,
            0.0,
            "minecraft:mangrove_swamp",
        );

        for ti in 0..5_usize {
            let temp = self.temperatures[ti];
            for hi in 0..5_usize {
                let hum = self.humidities[hi];
                let middle = self.pick_middle_biome(ti, hi, weird);
                let middle_or_badlands = self.pick_middle_biome_or_badlands_if_hot(ti, hi, weird);
                let middle_or_badlands_or_slope =
                    self.pick_middle_biome_or_badlands_if_hot_or_slope_if_cold(ti, hi, weird);
                let shattered = self.pick_shattered_biome(ti, hi, weird);
                let plateau = self.pick_plateau_biome(ti, hi, weird);
                let beach = self.pick_beach_biome(ti);
                let middle_or_windswept =
                    self.maybe_pick_windswept_savanna_biome(ti, hi, weird, middle);
                let shattered_coast = self.pick_shattered_coast_biome(ti, hi, weird);
                let slope = self.pick_slope_biome(ti, hi, weird);

                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    near_far,
                    self.erosions[0],
                    weird,
                    0.0,
                    slope,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    near_mid,
                    self.erosions[1],
                    weird,
                    0.0,
                    middle_or_badlands_or_slope,
                );
                let far_e1 = if ti == 0 { slope } else { plateau };
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.far_inland_cont,
                    self.erosions[1],
                    weird,
                    0.0,
                    far_e1,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.near_inland_cont,
                    self.erosions[2],
                    weird,
                    0.0,
                    middle,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.mid_inland_cont,
                    self.erosions[2],
                    weird,
                    0.0,
                    middle_or_badlands,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.far_inland_cont,
                    self.erosions[2],
                    weird,
                    0.0,
                    plateau,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    coast_near,
                    self.erosions[3],
                    weird,
                    0.0,
                    middle,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    mid_far,
                    self.erosions[3],
                    weird,
                    0.0,
                    middle_or_badlands,
                );
                if weird.max < 0 {
                    self.add_surface_biome(
                        entries,
                        temp,
                        hum,
                        self.coast_cont,
                        self.erosions[4],
                        weird,
                        0.0,
                        beach,
                    );
                    self.add_surface_biome(
                        entries,
                        temp,
                        hum,
                        near_far,
                        self.erosions[4],
                        weird,
                        0.0,
                        middle,
                    );
                } else {
                    self.add_surface_biome(
                        entries,
                        temp,
                        hum,
                        coast_far,
                        self.erosions[4],
                        weird,
                        0.0,
                        middle,
                    );
                }
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.coast_cont,
                    self.erosions[5],
                    weird,
                    0.0,
                    shattered_coast,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.near_inland_cont,
                    self.erosions[5],
                    weird,
                    0.0,
                    middle_or_windswept,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    mid_far,
                    self.erosions[5],
                    weird,
                    0.0,
                    shattered,
                );
                if weird.max < 0 {
                    self.add_surface_biome(
                        entries,
                        temp,
                        hum,
                        self.coast_cont,
                        self.erosions[6],
                        weird,
                        0.0,
                        beach,
                    );
                } else {
                    self.add_surface_biome(
                        entries,
                        temp,
                        hum,
                        self.coast_cont,
                        self.erosions[6],
                        weird,
                        0.0,
                        middle,
                    );
                }
                if ti == 0 {
                    self.add_surface_biome(
                        entries,
                        temp,
                        hum,
                        near_far,
                        self.erosions[6],
                        weird,
                        0.0,
                        middle,
                    );
                }
            }
        }
    }

    fn add_low_slice(&self, entries: &mut Vec<ClimateBiomeEntry>, weird: ClimateParameter) {
        let near_far = Self::p_span(self.near_inland_cont, self.far_inland_cont);
        let mid_far = Self::p_span(self.mid_inland_cont, self.far_inland_cont);
        let eros02 = Self::p_span(self.erosions[0], self.erosions[2]);
        let eros01 = Self::p_span(self.erosions[0], self.erosions[1]);
        let eros23 = Self::p_span(self.erosions[2], self.erosions[3]);
        let eros34 = Self::p_span(self.erosions[3], self.erosions[4]);
        let temp12 = Self::p_span(self.temperatures[1], self.temperatures[2]);
        let temp34 = Self::p_span(self.temperatures[3], self.temperatures[4]);

        self.add_surface_biome(
            entries,
            self.full_range,
            self.full_range,
            self.coast_cont,
            eros02,
            weird,
            0.0,
            "minecraft:stony_shore",
        );
        self.add_surface_biome(
            entries,
            temp12,
            self.full_range,
            near_far,
            self.erosions[6],
            weird,
            0.0,
            "minecraft:swamp",
        );
        self.add_surface_biome(
            entries,
            temp34,
            self.full_range,
            near_far,
            self.erosions[6],
            weird,
            0.0,
            "minecraft:mangrove_swamp",
        );

        for ti in 0..5_usize {
            let temp = self.temperatures[ti];
            for hi in 0..5_usize {
                let hum = self.humidities[hi];
                let middle = self.pick_middle_biome(ti, hi, weird);
                let middle_or_badlands = self.pick_middle_biome_or_badlands_if_hot(ti, hi, weird);
                let middle_or_badlands_or_slope =
                    self.pick_middle_biome_or_badlands_if_hot_or_slope_if_cold(ti, hi, weird);
                let beach = self.pick_beach_biome(ti);
                let middle_or_windswept =
                    self.maybe_pick_windswept_savanna_biome(ti, hi, weird, middle);
                let shattered_coast = self.pick_shattered_coast_biome(ti, hi, weird);

                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.near_inland_cont,
                    eros01,
                    weird,
                    0.0,
                    middle_or_badlands,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    mid_far,
                    eros01,
                    weird,
                    0.0,
                    middle_or_badlands_or_slope,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.near_inland_cont,
                    eros23,
                    weird,
                    0.0,
                    middle,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    mid_far,
                    eros23,
                    weird,
                    0.0,
                    middle_or_badlands,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.coast_cont,
                    eros34,
                    weird,
                    0.0,
                    beach,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    near_far,
                    self.erosions[4],
                    weird,
                    0.0,
                    middle,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.coast_cont,
                    self.erosions[5],
                    weird,
                    0.0,
                    shattered_coast,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.near_inland_cont,
                    self.erosions[5],
                    weird,
                    0.0,
                    middle_or_windswept,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    mid_far,
                    self.erosions[5],
                    weird,
                    0.0,
                    middle,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.coast_cont,
                    self.erosions[6],
                    weird,
                    0.0,
                    beach,
                );
                if ti == 0 {
                    self.add_surface_biome(
                        entries,
                        temp,
                        hum,
                        near_far,
                        self.erosions[6],
                        weird,
                        0.0,
                        middle,
                    );
                }
            }
        }
    }

    fn add_valleys(&self, entries: &mut Vec<ClimateBiomeEntry>, weird: ClimateParameter) {
        let coast_far = Self::p_span(self.coast_cont, self.far_inland_cont);
        let eros01 = Self::p_span(self.erosions[0], self.erosions[1]);
        let eros25 = Self::p_span(self.erosions[2], self.erosions[5]);
        let inland_far = Self::p_span(self.inland_cont, self.far_inland_cont);
        let mid_far = Self::p_span(self.mid_inland_cont, self.far_inland_cont);
        let temp12 = Self::p_span(self.temperatures[1], self.temperatures[2]);
        let temp34 = Self::p_span(self.temperatures[3], self.temperatures[4]);

        let frozen_stony = if weird.max < 0 {
            "minecraft:stony_shore"
        } else {
            "minecraft:frozen_river"
        };
        let unfrozen_stony = if weird.max < 0 {
            "minecraft:stony_shore"
        } else {
            "minecraft:river"
        };

        self.add_surface_biome(
            entries,
            self.frozen_range,
            self.full_range,
            self.coast_cont,
            eros01,
            weird,
            0.0,
            frozen_stony,
        );
        self.add_surface_biome(
            entries,
            self.unfrozen_range,
            self.full_range,
            self.coast_cont,
            eros01,
            weird,
            0.0,
            unfrozen_stony,
        );
        self.add_surface_biome(
            entries,
            self.frozen_range,
            self.full_range,
            self.near_inland_cont,
            eros01,
            weird,
            0.0,
            "minecraft:frozen_river",
        );
        self.add_surface_biome(
            entries,
            self.unfrozen_range,
            self.full_range,
            self.near_inland_cont,
            eros01,
            weird,
            0.0,
            "minecraft:river",
        );
        self.add_surface_biome(
            entries,
            self.frozen_range,
            self.full_range,
            coast_far,
            eros25,
            weird,
            0.0,
            "minecraft:frozen_river",
        );
        self.add_surface_biome(
            entries,
            self.unfrozen_range,
            self.full_range,
            coast_far,
            eros25,
            weird,
            0.0,
            "minecraft:river",
        );
        self.add_surface_biome(
            entries,
            self.frozen_range,
            self.full_range,
            self.coast_cont,
            self.erosions[6],
            weird,
            0.0,
            "minecraft:frozen_river",
        );
        self.add_surface_biome(
            entries,
            self.unfrozen_range,
            self.full_range,
            self.coast_cont,
            self.erosions[6],
            weird,
            0.0,
            "minecraft:river",
        );
        self.add_surface_biome(
            entries,
            temp12,
            self.full_range,
            inland_far,
            self.erosions[6],
            weird,
            0.0,
            "minecraft:swamp",
        );
        self.add_surface_biome(
            entries,
            temp34,
            self.full_range,
            inland_far,
            self.erosions[6],
            weird,
            0.0,
            "minecraft:mangrove_swamp",
        );
        self.add_surface_biome(
            entries,
            self.frozen_range,
            self.full_range,
            inland_far,
            self.erosions[6],
            weird,
            0.0,
            "minecraft:frozen_river",
        );

        for ti in 0..5_usize {
            let temp = self.temperatures[ti];
            for hi in 0..5_usize {
                let hum = self.humidities[hi];
                let middle_or_badlands = self.pick_middle_biome_or_badlands_if_hot(ti, hi, weird);
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    mid_far,
                    eros01,
                    weird,
                    0.0,
                    middle_or_badlands,
                );
            }
        }
    }

    fn add_inland_biomes(&self, entries: &mut Vec<ClimateBiomeEntry>) {
        self.add_mid_slice(entries, span(-1.0, -0.93333334));
        self.add_high_slice(entries, span(-0.93333334, -0.7666667));
        self.add_peaks(entries, span(-0.7666667, -0.56666666));
        self.add_high_slice(entries, span(-0.56666666, -0.4));
        self.add_mid_slice(entries, span(-0.4, -0.26666668));
        self.add_low_slice(entries, span(-0.26666668, -0.05));
        self.add_valleys(entries, span(-0.05, 0.05));
        self.add_low_slice(entries, span(0.05, 0.26666668));
        self.add_mid_slice(entries, span(0.26666668, 0.4));
        self.add_high_slice(entries, span(0.4, 0.56666666));
        self.add_peaks(entries, span(0.56666666, 0.7666667));
        self.add_high_slice(entries, span(0.7666667, 0.93333334));
        self.add_mid_slice(entries, span(0.93333334, 1.0));
    }

    fn add_off_coast_biomes(&self, entries: &mut Vec<ClimateBiomeEntry>) {
        self.add_surface_biome(
            entries,
            self.full_range,
            self.full_range,
            self.mushroom_fields_cont,
            self.full_range,
            self.full_range,
            0.0,
            "minecraft:mushroom_fields",
        );
        for ti in 0..5_usize {
            let temp = self.temperatures[ti];
            self.add_surface_biome(
                entries,
                temp,
                self.full_range,
                self.deep_ocean_cont,
                self.full_range,
                self.full_range,
                0.0,
                OCEANS[0][ti],
            );
            self.add_surface_biome(
                entries,
                temp,
                self.full_range,
                self.ocean_cont,
                self.full_range,
                self.full_range,
                0.0,
                OCEANS[1][ti],
            );
        }
    }

    fn add_underground_biomes(&self, entries: &mut Vec<ClimateBiomeEntry>) {
        self.add_underground_biome(
            entries,
            self.full_range,
            self.full_range,
            span(0.8, 1.0),
            self.full_range,
            self.full_range,
            0.0,
            "minecraft:dripstone_caves",
        );
        self.add_underground_biome(
            entries,
            self.full_range,
            span(0.7, 1.0),
            self.full_range,
            self.full_range,
            self.full_range,
            0.0,
            "minecraft:lush_caves",
        );
        self.add_bottom_biome(
            entries,
            self.full_range,
            self.full_range,
            self.full_range,
            Self::p_span(self.erosions[0], self.erosions[1]),
            self.full_range,
            0.0,
            "minecraft:deep_dark",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{
        biome_source_codec, biome_source_from_stem_id, builtin_biome, checkerboard_biome_source,
        climate_node_distance, climate_point, climate_sampler_sample, climate_target,
        multi_noise_parameter_list_preset, overworld_biome_parameters, parse_biome_json,
        parse_multi_noise_preset_json, quantize_coord, quart_to_block, select_biome_from_source,
        select_climate_biome, select_end_biome, unquantize_coord, BiomeGenerationSettings,
        BiomeSourceModel, ClimateBiomeEntry, ClimateParameter, ClimateParameterList,
        ClimateSamplerInput, ClimateTarget, MobCategory, BUILTIN_BIOMES, NETHER_BIOME_PARAMETERS,
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
        // Parameter list is generated at runtime by OverworldBiomeBuilder; verify its extent.
        assert!(overworld.parameters.len() > 100);
        assert!(overworld.used_biomes.contains(&"minecraft:mushroom_fields"));
        assert!(overworld.used_biomes.contains(&"minecraft:plains"));
        assert!(overworld.used_biomes.contains(&"minecraft:deep_dark"));
        assert!(multi_noise_parameter_list_preset("missing").is_none());
    }

    #[test]
    fn overworld_biome_parameters_builder_generates_complete_parameter_list() {
        let params = overworld_biome_parameters();
        // The builder generates hundreds of entries covering all slices, temperatures,
        // humidities, continentalnesses, erosions, and depths.
        assert!(
            params.len() > 100,
            "expected hundreds of entries, got {}",
            params.len()
        );

        // Lazy init is idempotent — same slice is returned every call.
        assert!(std::ptr::eq(params, overworld_biome_parameters()));

        // Mushroom fields occupies mushroomFieldsContinentalness (-1.2 to -1.05), which is
        // disjoint from all other continentalness ranges, so it unambiguously wins when
        // continentalness is anywhere in that interval.
        assert_eq!(
            select_climate_biome(params, climate_target(0.0, 0.0, -1.1, 0.0, 0.0, 0.0)),
            Some("minecraft:mushroom_fields")
        );

        // Underground biomes have depth=span(0.2, 0.9) or depth=point(1.1), making them
        // unambiguous winners when the depth target is in those ranges.
        //
        // Dripstone caves: cont=span(0.8,1.0), hum=full_range. Low humidity ensures
        // lush_caves (hum=span(0.7,1.0)) loses heavily on the humidity axis.
        assert_eq!(
            select_climate_biome(params, climate_target(0.0, -0.5, 0.9, 0.0, 0.5, 0.0)),
            Some("minecraft:dripstone_caves")
        );

        // Lush caves: hum=span(0.7,1.0), cont=full_range. High humidity and mid-depth.
        assert_eq!(
            select_climate_biome(params, climate_target(0.0, 0.9, 0.0, 0.0, 0.5, 0.0)),
            Some("minecraft:lush_caves")
        );

        // Deep dark: depth=point(1.1), eros=span(eros[0],eros[1]). The point(1.1) depth
        // is far from surface entries (distance≥1000) and underground span (distance=2000),
        // so deep_dark wins decisively when depth=1.1.
        assert_eq!(
            select_climate_biome(params, climate_target(0.0, 0.0, 0.0, -0.9, 1.1, 0.0)),
            Some("minecraft:deep_dark")
        );

        // All key biomes are reachable from the builder output.
        let biome_ids: std::collections::HashSet<_> = params.iter().map(|e| e.biome).collect();
        for expected in &[
            "minecraft:mushroom_fields",
            "minecraft:jagged_peaks",
            "minecraft:frozen_peaks",
            "minecraft:stony_peaks",
            "minecraft:snowy_slopes",
            "minecraft:grove",
            "minecraft:river",
            "minecraft:frozen_river",
            "minecraft:swamp",
            "minecraft:mangrove_swamp",
            "minecraft:deep_dark",
            "minecraft:dripstone_caves",
            "minecraft:lush_caves",
            "minecraft:windswept_savanna",
            "minecraft:cherry_grove",
            "minecraft:pale_garden",
        ] {
            assert!(
                biome_ids.contains(expected),
                "{expected} not found in overworld parameter list"
            );
        }
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

        // Verify climate_node_distance computes sum-of-squared-parameter-distances.
        let explicit_ps = climate_point(-0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.175).parameter_space();
        let expected_dist: i64 = explicit_ps
            .iter()
            .zip(target.parameter_array())
            .map(|(p, t)| {
                let d = p.distance(t);
                d * d
            })
            .sum();
        assert_eq!(
            climate_node_distance(explicit_ps, target.parameter_array()),
            expected_dist
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

    #[test]
    fn parse_multi_noise_preset_json_resolves_vanilla_data_files() {
        // Both vanilla JSON files just contain {"preset":"minecraft:X"}.
        let overworld = parse_multi_noise_preset_json(r#"{"preset":"minecraft:overworld"}"#);
        assert!(overworld.is_some());
        assert_eq!(overworld.unwrap().id, "minecraft:overworld");

        let nether = parse_multi_noise_preset_json(r#"{"preset":"minecraft:nether"}"#);
        assert!(nether.is_some());
        assert_eq!(nether.unwrap().id, "minecraft:nether");

        // Unknown preset returns None.
        assert!(parse_multi_noise_preset_json(r#"{"preset":"minecraft:unknown"}"#).is_none());
        // Malformed JSON returns None.
        assert!(parse_multi_noise_preset_json("{}").is_none());
        assert!(parse_multi_noise_preset_json("not json").is_none());
    }

    #[test]
    fn climate_rtree_find_nearest_agrees_with_bruteforce_on_overworld_params() {
        use super::ClimateParameterList;
        let params = overworld_biome_parameters();
        let list = ClimateParameterList::new(params.to_vec()).unwrap();

        // Test a variety of climate targets across the parameter space.
        let test_targets: &[ClimateTarget] = &[
            climate_target(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            climate_target(-0.8, -0.5, -1.1, 0.0, 0.0, 0.0), // mushroom fields
            climate_target(0.0, 0.0, 0.9, 0.0, 0.5, 0.0),    // dripstone caves
            climate_target(0.0, 0.9, 0.0, 0.0, 0.5, 0.0),    // lush caves
            climate_target(0.0, 0.0, 0.0, -0.9, 1.1, 0.0),   // deep dark
            climate_target(0.8, 0.8, 0.5, -0.5, 0.0, -0.4),  // warm inland
            climate_target(-0.8, -0.8, -0.5, 0.3, 0.0, 0.4), // cold ocean area
        ];

        for &target in test_targets {
            let bruteforce = list.find_value_bruteforce(target);
            let rtree = list.find_value_index(target);
            assert_eq!(
                bruteforce, rtree,
                "R-tree and brute-force disagree at {:?}: brute={bruteforce} rtree={rtree}",
                target
            );
        }
    }

    #[test]
    fn plains_biome_generation_and_mob_spawn_settings_match_vanilla_json() {
        // Load from the vanilla 26.1.2 data file.
        let json = std::fs::read_to_string(
            "../decompiled-server-26.1.2/data/minecraft/worldgen/biome/plains.json",
        )
        .expect("plains.json must be present in decompiled server data");

        let biome = parse_biome_json("minecraft:plains", &json)
            .expect("plains.json must parse without error");

        assert_eq!(biome.id, "minecraft:plains");
        assert!(biome.has_precipitation);
        assert!((biome.temperature - 0.8).abs() < 1e-4, "temperature should be 0.8");
        assert!((biome.downfall - 0.4).abs() < 1e-4, "downfall should be 0.4");

        // --- BiomeGenerationSettings ---

        let gen = &biome.generation_settings;

        // Exactly 11 GenerationStep.Decoration steps.
        assert_eq!(
            gen.features.len(),
            11,
            "features must have exactly 11 steps"
        );

        // Step 0 (RAW_GENERATION): empty for plains.
        assert!(
            gen.features[0].is_empty(),
            "RAW_GENERATION step should be empty for plains"
        );

        // Step 6 (UNDERGROUND_ORES): 29 entries including "minecraft:ore_dirt".
        assert_eq!(
            gen.features[6].len(),
            29,
            "UNDERGROUND_ORES step should have 29 entries"
        );
        assert!(
            gen.features[6].contains(&"minecraft:ore_dirt".to_string()),
            "UNDERGROUND_ORES step must include minecraft:ore_dirt"
        );

        // Step 9 (VEGETAL_DECORATION): contains "minecraft:trees_plains".
        assert!(
            gen.features[9].contains(&"minecraft:trees_plains".to_string()),
            "VEGETAL_DECORATION step must include minecraft:trees_plains"
        );

        // Carvers: "minecraft:cave" is present.
        assert!(
            gen.carvers.contains(&"minecraft:cave".to_string()),
            "carvers must include minecraft:cave"
        );

        // --- MobSpawnSettings ---

        let mob = &biome.mob_spawn_settings;

        // creature_spawn_probability: plains.json omits the key → default 0.1.
        assert!(
            (mob.creature_spawn_probability - 0.1).abs() < 1e-6,
            "creature_spawn_probability should be the default 0.1"
        );

        // Monster category: 9 entries; minecraft:spider has weight 100.
        let monsters = mob
            .spawners
            .get(&MobCategory::Monster)
            .expect("monster spawner category must be present");
        assert_eq!(monsters.len(), 9, "monster spawner list should have 9 entries");
        let spider = monsters
            .iter()
            .find(|e| e.entity_type == "minecraft:spider")
            .expect("minecraft:spider must be in monster spawners");
        assert_eq!(spider.weight, 100, "minecraft:spider weight must be 100");
        assert_eq!(spider.min_count, 4);
        assert_eq!(spider.max_count, 4);

        // spawn_costs: empty for plains.
        assert!(mob.spawn_costs.is_empty(), "plains spawn_costs should be empty");
    }
}
