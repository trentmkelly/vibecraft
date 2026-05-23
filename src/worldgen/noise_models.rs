use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoiseSettings {
    pub min_y: i32,
    pub height: i32,
    pub size_horizontal: i32,
    pub size_vertical: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NormalNoiseParameters {
    pub id: &'static str,
    pub first_octave: i32,
    pub amplitudes: &'static [f64],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalNoiseInstantiationPlan {
    pub id: &'static str,
    pub first_octave: i32,
    pub non_zero_octaves: Vec<i32>,
    pub use_new_initialization: bool,
    pub random: RandomSourceKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerlinNoiseConstructionPlan {
    pub first_octave: i32,
    pub octave_count: usize,
    pub zero_octave_index: i32,
    pub levels: Vec<PerlinNoiseLevelPlan>,
    pub legacy_created_zero_octave_first: bool,
    pub legacy_skipped_octaves: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerlinNoiseLevelPlan {
    pub level_index: usize,
    pub octave: i32,
    pub source: PerlinNoiseLevelSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerlinNoiseLevelSource {
    HashedOctave,
    SequentialLegacy,
    SequentialLegacyZero,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImprovedNoiseSnapshot {
    pub xo: f64,
    pub yo: f64,
    pub zo: f64,
    pub permutation: [u8; 256],
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerlinNoiseSnapshot {
    pub first_octave: i32,
    pub amplitudes: &'static [f64],
    pub levels: Vec<Option<ImprovedNoiseSnapshot>>,
    pub lowest_freq_input_factor: f64,
    pub lowest_freq_value_factor: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalNoiseSnapshot {
    pub parameters: NormalNoiseParameters,
    pub first: PerlinNoiseSnapshot,
    pub second: PerlinNoiseSnapshot,
    pub value_factor: f64,
    pub max_value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlendedNoiseSnapshot {
    pub min_limit_noise: PerlinNoiseSnapshot,
    pub max_limit_noise: PerlinNoiseSnapshot,
    pub main_noise: PerlinNoiseSnapshot,
    pub xz_multiplier: f64,
    pub y_multiplier: f64,
    pub xz_factor: f64,
    pub y_factor: f64,
    pub smear_scale_multiplier: f64,
    pub max_value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimplexNoiseSnapshot {
    pub xo: f64,
    pub yo: f64,
    pub zo: f64,
    pub permutation: [u8; 256],
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerlinSimplexNoiseSnapshot {
    /// Octave levels from highest-frequency (index 0) to lowest-frequency (last).
    /// Index `highFreqOctaves` corresponds to octave 0; positive octaves are at lower
    /// indices, negative octaves at higher indices — matching Java's noiseLevels layout.
    pub levels: Vec<Option<SimplexNoiseSnapshot>>,
    /// Scaling factor applied to the input coordinates for the first (highest-freq) octave.
    /// Equals 2^highFreqOctaves.
    pub highest_freq_input_factor: f64,
    /// Value weight for the first (highest-freq) octave. Equals 1 / (2^octaveCount - 1).
    pub highest_freq_value_factor: f64,
}

pub const SIMPLEX_GRADIENT: [[i32; 3]; 16] = [
    [1, 1, 0],
    [-1, 1, 0],
    [1, -1, 0],
    [-1, -1, 0],
    [1, 0, 1],
    [-1, 0, 1],
    [1, 0, -1],
    [-1, 0, -1],
    [0, 1, 1],
    [0, -1, 1],
    [0, 1, -1],
    [0, -1, -1],
    [1, 1, 0],
    [0, -1, 1],
    [-1, 1, 0],
    [0, -1, -1],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SynthNoiseSource {
    pub id: &'static str,
    pub codec: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoiseGeneratorSettings {
    pub id: &'static str,
    pub noise: NoiseSettings,
    pub default_block: &'static str,
    pub default_fluid: &'static str,
    pub noise_router: NoiseRouterPreset,
    pub surface_rule: SurfaceRulePreset,
    pub spawn_target: &'static [ClimateParameterPoint],
    pub sea_level: i32,
    pub disable_mob_generation: bool,
    pub aquifers_enabled: bool,
    pub ore_veins_enabled: bool,
    pub legacy_random_source: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoiseRouter {
    pub barrier: DensityFunction,
    pub fluid_level_floodedness: DensityFunction,
    pub fluid_level_spread: DensityFunction,
    pub lava: DensityFunction,
    pub temperature: DensityFunction,
    pub vegetation: DensityFunction,
    pub continents: DensityFunction,
    pub erosion: DensityFunction,
    pub depth: DensityFunction,
    pub ridges: DensityFunction,
    pub preliminary_surface_level: DensityFunction,
    pub final_density: DensityFunction,
    pub vein_toggle: DensityFunction,
    pub vein_ridged: DensityFunction,
    pub vein_gap: DensityFunction,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoiseRouterEntry {
    pub id: &'static str,
    pub router: NoiseRouter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseRouterPreset {
    Overworld { large_biomes: bool, amplified: bool },
    Nether,
    End,
    Caves,
    FloatingIslands,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceRulePreset {
    Overworld,
    Nether,
    End,
    OverworldLike {
        bedrock_roof: bool,
        bedrock_floor: bool,
        surface: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DensityFunction {
    Reference(&'static str),
    Constant(f64),
    YClampedGradient {
        from_y: i32,
        to_y: i32,
        from_value: f64,
        to_value: f64,
    },
    Clamp {
        input: &'static DensityFunction,
        min: f64,
        max: f64,
    },
    Mapped {
        kind: MappedDensityFunction,
        input: &'static DensityFunction,
    },
    Binary {
        kind: BinaryDensityFunction,
        argument1: &'static DensityFunction,
        argument2: &'static DensityFunction,
    },
    RangeChoice {
        input: &'static DensityFunction,
        min_inclusive: f64,
        max_exclusive: f64,
        when_in_range: &'static DensityFunction,
        when_out_of_range: &'static DensityFunction,
    },
    Marker {
        kind: DensityMarker,
        input: &'static DensityFunction,
    },
    Noise {
        noise: &'static str,
        xz_scale: f64,
        y_scale: f64,
    },
    ShiftA {
        noise: &'static str,
    },
    ShiftB {
        noise: &'static str,
    },
    Shift {
        noise: &'static str,
    },
    ShiftedNoise {
        shift_x: &'static DensityFunction,
        shift_y: &'static DensityFunction,
        shift_z: &'static DensityFunction,
        xz_scale: f64,
        y_scale: f64,
        noise: &'static str,
    },
    BlendedNoise {
        xz_scale: f64,
        y_scale: f64,
        xz_factor: f64,
        y_factor: f64,
        smear_scale_multiplier: f64,
    },
    EndIslands {
        seed: i64,
    },
    WeirdScaledSampler {
        input: &'static DensityFunction,
        noise: &'static str,
        rarity_mapper: RarityValueMapper,
    },
    BlendAlpha,
    BlendOffset,
    BlendDensity {
        input: &'static DensityFunction,
    },
    Beardifier,
    Spline {
        kind: TerrainSplineKind,
    },
    FindTopSurface {
        density: &'static DensityFunction,
        upper_bound: &'static DensityFunction,
        lower_bound: i32,
        cell_height: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedDensityFunction {
    Abs,
    Square,
    Cube,
    HalfNegative,
    QuarterNegative,
    Invert,
    Squeeze,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryDensityFunction {
    Add,
    Mul,
    Min,
    Max,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerrainSplineKind {
    OverworldOffset,
    OverworldFactor,
    OverworldJaggedness,
    OverworldLargeBiomesOffset,
    OverworldLargeBiomesFactor,
    OverworldLargeBiomesJaggedness,
    OverworldAmplifiedOffset,
    OverworldAmplifiedFactor,
    OverworldAmplifiedJaggedness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DensityMarker {
    Interpolated,
    FlatCache,
    Cache2D,
    CacheOnce,
    CacheAllInCell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RarityValueMapper {
    Type1,
    Type2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DensityFunctionType {
    pub id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DensityFunctionEntry {
    pub id: &'static str,
    pub function: DensityFunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoiseSettingsRegistryExpectation {
    pub id: &'static str,
    pub noise: NoiseSettings,
    pub default_block: &'static str,
    pub default_fluid: &'static str,
    pub router_id: &'static str,
    pub surface_rule: SurfaceRulePreset,
    pub spawn_target_len: usize,
    pub sea_level: i32,
    pub disable_mob_generation: bool,
    pub aquifers_enabled: bool,
    pub ore_veins_enabled: bool,
    pub legacy_random_source: bool,
}

