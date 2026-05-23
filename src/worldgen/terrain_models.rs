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

