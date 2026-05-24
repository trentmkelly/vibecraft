use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AquiferNoiseSettings {
    pub x_range: i32,
    pub y_range: i32,
    pub z_range: i32,
    pub x_separation: i32,
    pub y_separation: i32,
    pub z_separation: i32,
    pub x_spacing: i32,
    pub y_spacing: i32,
    pub z_spacing: i32,
    pub max_reasonable_distance_to_center: i32,
    pub sample_offset_x: i32,
    pub sample_offset_y: i32,
    pub sample_offset_z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FluidStatus {
    pub fluid_level: i32,
    pub fluid_type: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaveGenerationFamily {
    pub id: &'static str,
    pub noises: &'static [&'static str],
    pub output: CaveDensityOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveDensityOutput {
    CacheOnce,
    Clamp { min: i32, max: i32 },
    RangeChoice,
    Max,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OreVeinifierConstants {
    pub veininess_threshold: f64,
    pub edge_roundoff_begin: i32,
    pub max_edge_roundoff: f64,
    pub vein_solidness: f64,
    pub min_richness: f64,
    pub max_richness: f64,
    pub max_richness_threshold: f64,
    pub chance_of_raw_ore_block: f64,
    pub skip_ore_if_gap_noise_is_below: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OreVeinType {
    pub id: &'static str,
    pub ore: &'static str,
    pub raw_ore_block: &'static str,
    pub filler: &'static str,
    pub min_y: i32,
    pub max_y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OreVeinDecisionInput {
    pub y: i32,
    pub vein_toggle: f64,
    pub vein_ridged: f64,
    pub vein_gap: f64,
    pub solidness_random: f64,
    pub richness_random: f64,
    pub raw_ore_random: f64,
    pub debug_ore_veins: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfiguredCarver {
    pub id: &'static str,
    pub carver_type: WorldCarverType,
    pub probability: f32,
    pub y: HeightRange,
    pub y_scale: FloatProvider,
    pub lava_level: VerticalAnchor,
    pub debug: CarverDebugSettings,
    pub replaceable_tag: &'static str,
    pub shape: CarverShape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarverBlockInput {
    pub pos: BlockPos,
    pub block: &'static str,
    pub was_masked: bool,
    pub aquifer_state: Option<&'static str>,
    pub should_schedule_fluid_update: bool,
    pub debug_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarverBlockOutcome {
    pub pos: BlockPos,
    pub state: &'static str,
    pub mask_index: usize,
    pub mark_postprocessing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CarverSkipModel<'a> {
    None,
    Cave { floor_level: f64 },
    Canyon { width_factors: &'a [f32] },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaveTunnelStep {
    pub step: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub horizontal_radius: f64,
    pub vertical_radius: f64,
    pub can_reach: bool,
    pub carve: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaveTunnelBranch {
    pub split_step: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub left_thickness: f32,
    pub right_thickness: f32,
    pub left_horizontal_rotation: f32,
    pub right_horizontal_rotation: f32,
    pub vertical_rotation: f32,
    pub distance: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldCarverType {
    Cave,
    NetherCave,
    Canyon,
}

impl WorldCarverType {
    pub const fn id(self) -> &'static str {
        match self {
            WorldCarverType::Cave => "minecraft:cave",
            WorldCarverType::NetherCave => "minecraft:nether_cave",
            WorldCarverType::Canyon => "minecraft:canyon",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CarverShape {
    Cave {
        horizontal_radius_multiplier: FloatProvider,
        vertical_radius_multiplier: FloatProvider,
        floor_level: FloatProvider,
    },
    Canyon {
        vertical_rotation: FloatProvider,
        shape: CanyonShapeConfiguration,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanyonShapeConfiguration {
    pub distance_factor: FloatProvider,
    pub thickness: FloatProvider,
    pub width_smoothness: i32,
    pub horizontal_radius_factor: FloatProvider,
    pub vertical_radius_default_factor: f32,
    pub vertical_radius_center_factor: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FloatProvider {
    Constant(f32),
    Uniform { min: f32, max: f32 },
    Trapezoid { min: f32, max: f32, plateau: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeightRange {
    pub min: VerticalAnchor,
    pub max: VerticalAnchor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAnchor {
    Absolute(i32),
    AboveBottom(i32),
    BelowTop(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldGenerationHeightContext {
    pub min_y: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeightProviderType {
    pub id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeightedHeightProvider {
    pub weight: i32,
    pub provider: HeightProvider,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightProvider {
    Constant {
        value: VerticalAnchor,
    },
    Uniform {
        min_inclusive: VerticalAnchor,
        max_inclusive: VerticalAnchor,
    },
    BiasedToBottom {
        min_inclusive: VerticalAnchor,
        max_inclusive: VerticalAnchor,
        inner: i32,
    },
    VeryBiasedToBottom {
        min_inclusive: VerticalAnchor,
        max_inclusive: VerticalAnchor,
        inner: i32,
    },
    Trapezoid {
        min_inclusive: VerticalAnchor,
        max_inclusive: VerticalAnchor,
        plateau: i32,
    },
    WeightedList {
        distribution: &'static [WeightedHeightProvider],
    },
}

pub const AQUIFER_NOISE_SETTINGS: AquiferNoiseSettings = AquiferNoiseSettings {
    x_range: 10,
    y_range: 9,
    z_range: 10,
    x_separation: 6,
    y_separation: 3,
    z_separation: 6,
    x_spacing: 16,
    y_spacing: 12,
    z_spacing: 16,
    max_reasonable_distance_to_center: 11,
    sample_offset_x: -5,
    sample_offset_y: 1,
    sample_offset_z: -5,
};

pub const AQUIFER_SURFACE_SAMPLING_OFFSETS_IN_CHUNKS: &[(i32, i32)] = &[
    (0, 0),
    (-2, -1),
    (-1, -1),
    (0, -1),
    (1, -1),
    (-3, 0),
    (-2, 0),
    (-1, 0),
    (1, 0),
    (-2, 1),
    (-1, 1),
    (0, 1),
    (1, 1),
];

pub const CAVE_GENERATION_FAMILIES: &[CaveGenerationFamily] = &[
    CaveGenerationFamily {
        id: "minecraft:overworld/caves/spaghetti_roughness_function",
        noises: &[
            "minecraft:spaghetti_roughness",
            "minecraft:spaghetti_roughness_modulator",
        ],
        output: CaveDensityOutput::CacheOnce,
    },
    CaveGenerationFamily {
        id: "minecraft:overworld/caves/entrances",
        noises: &[
            "minecraft:spaghetti_3d_rarity",
            "minecraft:spaghetti_3d_thickness",
            "minecraft:spaghetti_3d_1",
            "minecraft:spaghetti_3d_2",
            "minecraft:cave_entrance",
        ],
        output: CaveDensityOutput::CacheOnce,
    },
    CaveGenerationFamily {
        id: "minecraft:overworld/caves/noodle",
        noises: &[
            "minecraft:noodle",
            "minecraft:noodle_thickness",
            "minecraft:noodle_ridge_a",
            "minecraft:noodle_ridge_b",
        ],
        output: CaveDensityOutput::RangeChoice,
    },
    CaveGenerationFamily {
        id: "minecraft:overworld/caves/pillars",
        noises: &[
            "minecraft:pillar",
            "minecraft:pillar_rareness",
            "minecraft:pillar_thickness",
        ],
        output: CaveDensityOutput::CacheOnce,
    },
    CaveGenerationFamily {
        id: "minecraft:overworld/caves/spaghetti_2d",
        noises: &[
            "minecraft:spaghetti_2d_modulator",
            "minecraft:spaghetti_2d",
            "minecraft:spaghetti_2d_elevation",
            "minecraft:spaghetti_2d_thickness",
        ],
        output: CaveDensityOutput::Clamp { min: -1, max: 1 },
    },
    CaveGenerationFamily {
        id: "minecraft:overworld/caves/underground",
        noises: &["minecraft:cave_layer", "minecraft:cave_cheese"],
        output: CaveDensityOutput::Max,
    },
];

pub const ORE_VEINIFIER_CONSTANTS: OreVeinifierConstants = OreVeinifierConstants {
    veininess_threshold: 0.4,
    edge_roundoff_begin: 20,
    max_edge_roundoff: 0.2,
    vein_solidness: 0.7,
    min_richness: 0.1,
    max_richness: 0.3,
    max_richness_threshold: 0.6,
    chance_of_raw_ore_block: 0.02,
    skip_ore_if_gap_noise_is_below: -0.3,
};

pub const ORE_VEIN_TYPES: &[OreVeinType] = &[
    OreVeinType {
        id: "copper",
        ore: "minecraft:copper_ore",
        raw_ore_block: "minecraft:raw_copper_block",
        filler: "minecraft:granite",
        min_y: 0,
        max_y: 50,
    },
    OreVeinType {
        id: "iron",
        ore: "minecraft:deepslate_iron_ore",
        raw_ore_block: "minecraft:raw_iron_block",
        filler: "minecraft:tuff",
        min_y: -60,
        max_y: -8,
    },
];

pub const WORLD_CARVER_TYPES: &[WorldCarverType] = &[
    WorldCarverType::Cave,
    WorldCarverType::NetherCave,
    WorldCarverType::Canyon,
];

pub const CONFIGURED_CARVERS: &[ConfiguredCarver] = &[
    ConfiguredCarver {
        id: "minecraft:cave",
        carver_type: WorldCarverType::Cave,
        probability: 0.15,
        y: HeightRange {
            min: VerticalAnchor::AboveBottom(8),
            max: VerticalAnchor::Absolute(180),
        },
        y_scale: FloatProvider::Uniform { min: 0.1, max: 0.9 },
        lava_level: VerticalAnchor::AboveBottom(8),
        debug: CarverDebugSettings {
            enabled: false,
            barrier_state: "minecraft:crimson_button",
        },
        replaceable_tag: "#minecraft:overworld_carver_replaceables",
        shape: CarverShape::Cave {
            horizontal_radius_multiplier: FloatProvider::Uniform { min: 0.7, max: 1.4 },
            vertical_radius_multiplier: FloatProvider::Uniform { min: 0.8, max: 1.3 },
            floor_level: FloatProvider::Uniform {
                min: -1.0,
                max: -0.4,
            },
        },
    },
    ConfiguredCarver {
        id: "minecraft:cave_extra_underground",
        carver_type: WorldCarverType::Cave,
        probability: 0.07,
        y: HeightRange {
            min: VerticalAnchor::AboveBottom(8),
            max: VerticalAnchor::Absolute(47),
        },
        y_scale: FloatProvider::Uniform { min: 0.1, max: 0.9 },
        lava_level: VerticalAnchor::AboveBottom(8),
        debug: CarverDebugSettings {
            enabled: false,
            barrier_state: "minecraft:oak_button",
        },
        replaceable_tag: "#minecraft:overworld_carver_replaceables",
        shape: CarverShape::Cave {
            horizontal_radius_multiplier: FloatProvider::Uniform { min: 0.7, max: 1.4 },
            vertical_radius_multiplier: FloatProvider::Uniform { min: 0.8, max: 1.3 },
            floor_level: FloatProvider::Uniform {
                min: -1.0,
                max: -0.4,
            },
        },
    },
    ConfiguredCarver {
        id: "minecraft:canyon",
        carver_type: WorldCarverType::Canyon,
        probability: 0.01,
        y: HeightRange {
            min: VerticalAnchor::Absolute(10),
            max: VerticalAnchor::Absolute(67),
        },
        y_scale: FloatProvider::Constant(3.0),
        lava_level: VerticalAnchor::AboveBottom(8),
        debug: CarverDebugSettings {
            enabled: false,
            barrier_state: "minecraft:warped_button",
        },
        replaceable_tag: "#minecraft:overworld_carver_replaceables",
        shape: CarverShape::Canyon {
            vertical_rotation: FloatProvider::Uniform {
                min: -0.125,
                max: 0.125,
            },
            shape: CanyonShapeConfiguration {
                distance_factor: FloatProvider::Uniform {
                    min: 0.75,
                    max: 1.0,
                },
                thickness: FloatProvider::Trapezoid {
                    min: 0.0,
                    max: 6.0,
                    plateau: 2.0,
                },
                width_smoothness: 3,
                horizontal_radius_factor: FloatProvider::Uniform {
                    min: 0.75,
                    max: 1.0,
                },
                vertical_radius_default_factor: 1.0,
                vertical_radius_center_factor: 0.0,
            },
        },
    },
    ConfiguredCarver {
        id: "minecraft:nether_cave",
        carver_type: WorldCarverType::NetherCave,
        probability: 0.2,
        y: HeightRange {
            min: VerticalAnchor::Absolute(0),
            max: VerticalAnchor::BelowTop(1),
        },
        y_scale: FloatProvider::Constant(0.5),
        lava_level: VerticalAnchor::AboveBottom(10),
        debug: CarverDebugSettings {
            enabled: false,
            barrier_state: "minecraft:air",
        },
        replaceable_tag: "#minecraft:nether_carver_replaceables",
        shape: CarverShape::Cave {
            horizontal_radius_multiplier: FloatProvider::Constant(1.0),
            vertical_radius_multiplier: FloatProvider::Constant(1.0),
            floor_level: FloatProvider::Constant(-0.7),
        },
    },
];
