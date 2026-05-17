#![allow(dead_code)]

use crate::biome::{span, ClimateParameterPoint};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoiseSettings {
    pub min_y: i32,
    pub height: i32,
    pub size_horizontal: i32,
    pub size_vertical: i32,
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
    Marker {
        kind: DensityMarker,
        input: &'static DensityFunction,
    },
    Noise {
        noise: &'static str,
        xz_scale: f64,
        y_scale: f64,
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
    Spline,
    FindTopSurface,
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

pub const OVERWORLD_NOISE_SETTINGS: NoiseSettings = NoiseSettings::new(-64, 384, 1, 2);
pub const NETHER_NOISE_SETTINGS: NoiseSettings = NoiseSettings::new(0, 128, 1, 2);
pub const END_NOISE_SETTINGS: NoiseSettings = NoiseSettings::new(0, 128, 2, 1);
pub const CAVES_NOISE_SETTINGS: NoiseSettings = NoiseSettings::new(-64, 192, 1, 2);
pub const FLOATING_ISLANDS_NOISE_SETTINGS: NoiseSettings = NoiseSettings::new(0, 256, 2, 1);

pub const OVERWORLD_SPAWN_TARGET: &[ClimateParameterPoint] = &[
    ClimateParameterPoint {
        temperature: span(-1.0, 1.0),
        humidity: span(-1.0, 1.0),
        continentalness: span(-0.11, 1.0),
        erosion: span(-1.0, 1.0),
        depth: span(0.0, 0.0),
        weirdness: span(-1.0, -0.16),
        offset: 0,
    },
    ClimateParameterPoint {
        temperature: span(-1.0, 1.0),
        humidity: span(-1.0, 1.0),
        continentalness: span(-0.11, 1.0),
        erosion: span(-1.0, 1.0),
        depth: span(0.0, 0.0),
        weirdness: span(0.16, 1.0),
        offset: 0,
    },
];

pub const BUILTIN_NOISE_GENERATOR_SETTINGS: &[NoiseGeneratorSettings] = &[
    NoiseGeneratorSettings {
        id: "minecraft:overworld",
        noise: OVERWORLD_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        noise_router: NoiseRouterPreset::Overworld {
            large_biomes: false,
            amplified: false,
        },
        surface_rule: SurfaceRulePreset::Overworld,
        spawn_target: OVERWORLD_SPAWN_TARGET,
        sea_level: 63,
        disable_mob_generation: false,
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
    },
    NoiseGeneratorSettings {
        id: "minecraft:large_biomes",
        noise: OVERWORLD_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        noise_router: NoiseRouterPreset::Overworld {
            large_biomes: true,
            amplified: false,
        },
        surface_rule: SurfaceRulePreset::Overworld,
        spawn_target: OVERWORLD_SPAWN_TARGET,
        sea_level: 63,
        disable_mob_generation: false,
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
    },
    NoiseGeneratorSettings {
        id: "minecraft:amplified",
        noise: OVERWORLD_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        noise_router: NoiseRouterPreset::Overworld {
            large_biomes: false,
            amplified: true,
        },
        surface_rule: SurfaceRulePreset::Overworld,
        spawn_target: OVERWORLD_SPAWN_TARGET,
        sea_level: 63,
        disable_mob_generation: false,
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
    },
    NoiseGeneratorSettings {
        id: "minecraft:nether",
        noise: NETHER_NOISE_SETTINGS,
        default_block: "minecraft:netherrack",
        default_fluid: "minecraft:lava",
        noise_router: NoiseRouterPreset::Nether,
        surface_rule: SurfaceRulePreset::Nether,
        spawn_target: &[],
        sea_level: 32,
        disable_mob_generation: false,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
    NoiseGeneratorSettings {
        id: "minecraft:end",
        noise: END_NOISE_SETTINGS,
        default_block: "minecraft:end_stone",
        default_fluid: "minecraft:air",
        noise_router: NoiseRouterPreset::End,
        surface_rule: SurfaceRulePreset::End,
        spawn_target: &[],
        sea_level: 0,
        disable_mob_generation: true,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
    NoiseGeneratorSettings {
        id: "minecraft:caves",
        noise: CAVES_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        noise_router: NoiseRouterPreset::Caves,
        surface_rule: SurfaceRulePreset::OverworldLike {
            bedrock_roof: false,
            bedrock_floor: true,
            surface: true,
        },
        spawn_target: &[],
        sea_level: 32,
        disable_mob_generation: false,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
    NoiseGeneratorSettings {
        id: "minecraft:floating_islands",
        noise: FLOATING_ISLANDS_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        noise_router: NoiseRouterPreset::FloatingIslands,
        surface_rule: SurfaceRulePreset::OverworldLike {
            bedrock_roof: false,
            bedrock_floor: false,
            surface: false,
        },
        spawn_target: &[],
        sea_level: -64,
        disable_mob_generation: false,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
];

pub const ZERO_DENSITY: DensityFunction = DensityFunction::Constant(0.0);
pub const Y_DENSITY: DensityFunction = DensityFunction::YClampedGradient {
    from_y: -4064,
    to_y: 4062,
    from_value: -4064.0,
    to_value: 4062.0,
};
pub const SHIFT_X_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::FlatCache,
    input: &SHIFT_X_CACHE_2D_DENSITY,
};
pub const SHIFT_X_CACHE_2D_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Cache2D,
    input: &SHIFT_A_DENSITY,
};
pub const SHIFT_A_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:shift",
    xz_scale: 0.25,
    y_scale: 0.0,
};
pub const SHIFT_Z_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::FlatCache,
    input: &SHIFT_Z_CACHE_2D_DENSITY,
};
pub const SHIFT_Z_CACHE_2D_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Cache2D,
    input: &SHIFT_B_DENSITY,
};
pub const SHIFT_B_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:shift",
    xz_scale: 0.25,
    y_scale: 0.0,
};
pub const BASE_3D_NOISE_OVERWORLD_DENSITY: DensityFunction = DensityFunction::BlendedNoise {
    xz_scale: 0.25,
    y_scale: 0.125,
    xz_factor: 80.0,
    y_factor: 160.0,
    smear_scale_multiplier: 8.0,
};
pub const BASE_3D_NOISE_NETHER_DENSITY: DensityFunction = DensityFunction::BlendedNoise {
    xz_scale: 0.25,
    y_scale: 0.375,
    xz_factor: 80.0,
    y_factor: 60.0,
    smear_scale_multiplier: 8.0,
};
pub const BASE_3D_NOISE_END_DENSITY: DensityFunction = DensityFunction::BlendedNoise {
    xz_scale: 0.25,
    y_scale: 0.25,
    xz_factor: 80.0,
    y_factor: 160.0,
    smear_scale_multiplier: 4.0,
};

pub const DENSITY_FUNCTION_TYPES: &[DensityFunctionType] = &[
    DensityFunctionType { id: "blend_alpha" },
    DensityFunctionType { id: "blend_offset" },
    DensityFunctionType { id: "beardifier" },
    DensityFunctionType {
        id: "old_blended_noise",
    },
    DensityFunctionType { id: "interpolated" },
    DensityFunctionType { id: "flat_cache" },
    DensityFunctionType { id: "cache_2d" },
    DensityFunctionType { id: "cache_once" },
    DensityFunctionType {
        id: "cache_all_in_cell",
    },
    DensityFunctionType { id: "noise" },
    DensityFunctionType { id: "end_islands" },
    DensityFunctionType {
        id: "weird_scaled_sampler",
    },
    DensityFunctionType {
        id: "shifted_noise",
    },
    DensityFunctionType { id: "range_choice" },
    DensityFunctionType { id: "shift_a" },
    DensityFunctionType { id: "shift_b" },
    DensityFunctionType { id: "shift" },
    DensityFunctionType {
        id: "blend_density",
    },
    DensityFunctionType { id: "clamp" },
    DensityFunctionType { id: "abs" },
    DensityFunctionType { id: "square" },
    DensityFunctionType { id: "cube" },
    DensityFunctionType {
        id: "half_negative",
    },
    DensityFunctionType {
        id: "quarter_negative",
    },
    DensityFunctionType { id: "invert" },
    DensityFunctionType { id: "squeeze" },
    DensityFunctionType { id: "add" },
    DensityFunctionType { id: "mul" },
    DensityFunctionType { id: "min" },
    DensityFunctionType { id: "max" },
    DensityFunctionType { id: "spline" },
    DensityFunctionType { id: "constant" },
    DensityFunctionType {
        id: "y_clamped_gradient",
    },
    DensityFunctionType {
        id: "find_top_surface",
    },
];

pub const BUILTIN_DENSITY_FUNCTIONS: &[DensityFunctionEntry] = &[
    DensityFunctionEntry {
        id: "minecraft:zero",
        function: ZERO_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:y",
        function: Y_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:shift_x",
        function: SHIFT_X_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:shift_z",
        function: SHIFT_Z_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/base_3d_noise",
        function: BASE_3D_NOISE_OVERWORLD_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:nether/base_3d_noise",
        function: BASE_3D_NOISE_NETHER_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:end/base_3d_noise",
        function: BASE_3D_NOISE_END_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/continents",
        function: DensityFunction::ShiftedNoise {
            shift_x: &SHIFT_X_DENSITY,
            shift_y: &ZERO_DENSITY,
            shift_z: &SHIFT_Z_DENSITY,
            xz_scale: 0.25,
            y_scale: 0.0,
            noise: "minecraft:continentalness",
        },
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/erosion",
        function: DensityFunction::ShiftedNoise {
            shift_x: &SHIFT_X_DENSITY,
            shift_y: &ZERO_DENSITY,
            shift_z: &SHIFT_Z_DENSITY,
            xz_scale: 0.25,
            y_scale: 0.0,
            noise: "minecraft:erosion",
        },
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/ridges",
        function: DensityFunction::ShiftedNoise {
            shift_x: &SHIFT_X_DENSITY,
            shift_y: &ZERO_DENSITY,
            shift_z: &SHIFT_Z_DENSITY,
            xz_scale: 0.25,
            y_scale: 0.0,
            noise: "minecraft:ridge",
        },
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/ridges_folded",
        function: DensityFunction::Mapped {
            kind: MappedDensityFunction::Abs,
            input: &RIDGE_FOLD_SOURCE_DENSITY,
        },
    },
    DensityFunctionEntry {
        id: "minecraft:overworld_large_biomes/continents",
        function: DensityFunction::ShiftedNoise {
            shift_x: &SHIFT_X_DENSITY,
            shift_y: &ZERO_DENSITY,
            shift_z: &SHIFT_Z_DENSITY,
            xz_scale: 0.25,
            y_scale: 0.0,
            noise: "minecraft:continentalness_large",
        },
    },
    DensityFunctionEntry {
        id: "minecraft:overworld_large_biomes/erosion",
        function: DensityFunction::ShiftedNoise {
            shift_x: &SHIFT_X_DENSITY,
            shift_y: &ZERO_DENSITY,
            shift_z: &SHIFT_Z_DENSITY,
            xz_scale: 0.25,
            y_scale: 0.0,
            noise: "minecraft:erosion_large",
        },
    },
    DensityFunctionEntry {
        id: "minecraft:end/sloped_cheese",
        function: DensityFunction::Binary {
            kind: BinaryDensityFunction::Add,
            argument1: &END_ISLANDS_DENSITY,
            argument2: &BASE_3D_NOISE_END_DENSITY,
        },
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/caves/spaghetti_2d_thickness_modulator",
        function: DensityFunction::Marker {
            kind: DensityMarker::CacheOnce,
            input: &SPAGHETTI_2D_THICKNESS_MODULATOR_DENSITY,
        },
    },
];

pub const RIDGE_FOLD_SOURCE_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &RIDGE_SCALE_DENSITY,
    argument2: &RIDGE_OFFSET_DENSITY,
};
pub const RIDGE_SCALE_DENSITY: DensityFunction = DensityFunction::Constant(-3.0);
pub const RIDGE_OFFSET_DENSITY: DensityFunction = DensityFunction::Constant(2.0);
pub const END_ISLANDS_DENSITY: DensityFunction = DensityFunction::EndIslands { seed: 0 };
pub const SPAGHETTI_2D_THICKNESS_MODULATOR_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &SPAGHETTI_2D_THICKNESS_MID_DENSITY,
    argument2: &SPAGHETTI_2D_THICKNESS_SCALED_DENSITY,
};
pub const SPAGHETTI_2D_THICKNESS_MID_DENSITY: DensityFunction = DensityFunction::Constant(-0.95);
pub const SPAGHETTI_2D_THICKNESS_SCALED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &SPAGHETTI_2D_THICKNESS_FACTOR_DENSITY,
    argument2: &SPAGHETTI_2D_THICKNESS_NOISE_DENSITY,
};
pub const SPAGHETTI_2D_THICKNESS_FACTOR_DENSITY: DensityFunction = DensityFunction::Constant(-0.35);
pub const SPAGHETTI_2D_THICKNESS_NOISE_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:spaghetti_2d_thickness",
    xz_scale: 2.0,
    y_scale: 1.0,
};
pub const TEST_NEGATIVE_DENSITY: DensityFunction = DensityFunction::Constant(-2.0);
pub const TEST_POSITIVE_DENSITY: DensityFunction = DensityFunction::Constant(3.0);

pub const OVERWORLD_NOISE_ROUTER: NoiseRouter = NoiseRouter {
    barrier: DensityFunction::Noise {
        noise: "minecraft:aquifer_barrier",
        xz_scale: 1.0,
        y_scale: 0.5,
    },
    fluid_level_floodedness: DensityFunction::Noise {
        noise: "minecraft:aquifer_fluid_level_floodedness",
        xz_scale: 1.0,
        y_scale: 0.67,
    },
    fluid_level_spread: DensityFunction::Noise {
        noise: "minecraft:aquifer_fluid_level_spread",
        xz_scale: 1.0,
        y_scale: 0.7142857142857143,
    },
    lava: DensityFunction::Noise {
        noise: "minecraft:aquifer_lava",
        xz_scale: 1.0,
        y_scale: 1.0,
    },
    temperature: DensityFunction::ShiftedNoise {
        shift_x: &SHIFT_X_DENSITY,
        shift_y: &ZERO_DENSITY,
        shift_z: &SHIFT_Z_DENSITY,
        xz_scale: 0.25,
        y_scale: 0.0,
        noise: "minecraft:temperature",
    },
    vegetation: DensityFunction::ShiftedNoise {
        shift_x: &SHIFT_X_DENSITY,
        shift_y: &ZERO_DENSITY,
        shift_z: &SHIFT_Z_DENSITY,
        xz_scale: 0.25,
        y_scale: 0.0,
        noise: "minecraft:vegetation",
    },
    continents: DensityFunction::Reference("minecraft:overworld/continents"),
    erosion: DensityFunction::Reference("minecraft:overworld/erosion"),
    depth: DensityFunction::Reference("minecraft:overworld/depth"),
    ridges: DensityFunction::Reference("minecraft:overworld/ridges"),
    preliminary_surface_level: DensityFunction::Reference(
        "minecraft:overworld/preliminary_surface_level",
    ),
    final_density: DensityFunction::Reference("minecraft:overworld/final_density"),
    vein_toggle: DensityFunction::Noise {
        noise: "minecraft:ore_veininess",
        xz_scale: 1.5,
        y_scale: 1.5,
    },
    vein_ridged: DensityFunction::Reference("minecraft:overworld/vein_ridged"),
    vein_gap: DensityFunction::Noise {
        noise: "minecraft:ore_gap",
        xz_scale: 1.0,
        y_scale: 1.0,
    },
};

pub const LARGE_BIOMES_NOISE_ROUTER: NoiseRouter = NoiseRouter {
    temperature: DensityFunction::ShiftedNoise {
        shift_x: &SHIFT_X_DENSITY,
        shift_y: &ZERO_DENSITY,
        shift_z: &SHIFT_Z_DENSITY,
        xz_scale: 0.25,
        y_scale: 0.0,
        noise: "minecraft:temperature_large",
    },
    vegetation: DensityFunction::ShiftedNoise {
        shift_x: &SHIFT_X_DENSITY,
        shift_y: &ZERO_DENSITY,
        shift_z: &SHIFT_Z_DENSITY,
        xz_scale: 0.25,
        y_scale: 0.0,
        noise: "minecraft:vegetation_large",
    },
    continents: DensityFunction::Reference("minecraft:overworld_large_biomes/continents"),
    erosion: DensityFunction::Reference("minecraft:overworld_large_biomes/erosion"),
    depth: DensityFunction::Reference("minecraft:overworld_large_biomes/depth"),
    preliminary_surface_level: DensityFunction::Reference(
        "minecraft:overworld_large_biomes/preliminary_surface_level",
    ),
    final_density: DensityFunction::Reference("minecraft:overworld_large_biomes/final_density"),
    ..OVERWORLD_NOISE_ROUTER
};

pub const AMPLIFIED_NOISE_ROUTER: NoiseRouter = NoiseRouter {
    depth: DensityFunction::Reference("minecraft:overworld_amplified/depth"),
    preliminary_surface_level: DensityFunction::Reference(
        "minecraft:overworld_amplified/preliminary_surface_level",
    ),
    final_density: DensityFunction::Reference("minecraft:overworld_amplified/final_density"),
    ..OVERWORLD_NOISE_ROUTER
};

pub const NETHER_NOISE_ROUTER: NoiseRouter = NoiseRouter {
    temperature: DensityFunction::ShiftedNoise {
        shift_x: &ZERO_DENSITY,
        shift_y: &ZERO_DENSITY,
        shift_z: &ZERO_DENSITY,
        xz_scale: 0.25,
        y_scale: 0.0,
        noise: "minecraft:temperature_nether",
    },
    vegetation: DensityFunction::ShiftedNoise {
        shift_x: &ZERO_DENSITY,
        shift_y: &ZERO_DENSITY,
        shift_z: &ZERO_DENSITY,
        xz_scale: 0.25,
        y_scale: 0.0,
        noise: "minecraft:vegetation_nether",
    },
    final_density: DensityFunction::Reference("minecraft:nether/final_density"),
    ..NoiseRouter::simple(ZERO_DENSITY)
};

pub const CAVES_NOISE_ROUTER: NoiseRouter =
    NoiseRouter::simple(DensityFunction::Reference("minecraft:caves/final_density"));
pub const FLOATING_ISLANDS_NOISE_ROUTER: NoiseRouter = NoiseRouter::simple(
    DensityFunction::Reference("minecraft:floating_islands/final_density"),
);
pub const END_NOISE_ROUTER: NoiseRouter = NoiseRouter {
    erosion: DensityFunction::Marker {
        kind: DensityMarker::Cache2D,
        input: &END_ISLANDS_DENSITY,
    },
    final_density: DensityFunction::Reference("minecraft:end/final_density"),
    ..NoiseRouter::simple(ZERO_DENSITY)
};
pub const NONE_NOISE_ROUTER: NoiseRouter = NoiseRouter::simple(ZERO_DENSITY);

pub const BUILTIN_NOISE_ROUTERS: &[NoiseRouterEntry] = &[
    NoiseRouterEntry {
        id: "minecraft:overworld",
        router: OVERWORLD_NOISE_ROUTER,
    },
    NoiseRouterEntry {
        id: "minecraft:large_biomes",
        router: LARGE_BIOMES_NOISE_ROUTER,
    },
    NoiseRouterEntry {
        id: "minecraft:amplified",
        router: AMPLIFIED_NOISE_ROUTER,
    },
    NoiseRouterEntry {
        id: "minecraft:nether",
        router: NETHER_NOISE_ROUTER,
    },
    NoiseRouterEntry {
        id: "minecraft:end",
        router: END_NOISE_ROUTER,
    },
    NoiseRouterEntry {
        id: "minecraft:caves",
        router: CAVES_NOISE_ROUTER,
    },
    NoiseRouterEntry {
        id: "minecraft:floating_islands",
        router: FLOATING_ISLANDS_NOISE_ROUTER,
    },
    NoiseRouterEntry {
        id: "minecraft:none",
        router: NONE_NOISE_ROUTER,
    },
];

impl NoiseSettings {
    pub const fn new(min_y: i32, height: i32, size_horizontal: i32, size_vertical: i32) -> Self {
        Self {
            min_y,
            height,
            size_horizontal,
            size_vertical,
        }
    }

    pub fn validate(self) -> Result<(), String> {
        if self.min_y + self.height > 2032 {
            return Err("min_y + height cannot be higher than: 2032".to_string());
        }
        if self.height % 16 != 0 {
            return Err("height has to be a multiple of 16".to_string());
        }
        if self.min_y % 16 != 0 {
            return Err("min_y has to be a multiple of 16".to_string());
        }
        if !(1..=4).contains(&self.size_horizontal) {
            return Err("size_horizontal must be in 1..=4".to_string());
        }
        if !(1..=4).contains(&self.size_vertical) {
            return Err("size_vertical must be in 1..=4".to_string());
        }
        Ok(())
    }

    pub fn cell_height(self) -> i32 {
        self.size_vertical * 4
    }

    pub fn cell_width(self) -> i32 {
        self.size_horizontal * 4
    }

    pub fn clamp_to_height(self, min_y: i32, max_y: i32) -> Self {
        let new_min_y = self.min_y.max(min_y);
        let new_height = (self.min_y + self.height).min(max_y + 1) - new_min_y;
        Self::new(
            new_min_y,
            new_height,
            self.size_horizontal,
            self.size_vertical,
        )
    }
}

pub fn builtin_noise_generator_settings(id: &str) -> Option<&'static NoiseGeneratorSettings> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    BUILTIN_NOISE_GENERATOR_SETTINGS.iter().find(|settings| {
        settings
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|settings_name| settings_name == name)
    })
}

impl DensityFunction {
    pub fn compute(self, block_y: i32) -> f64 {
        match self {
            DensityFunction::Reference(id) => builtin_density_function(id)
                .map(|entry| entry.function.compute(block_y))
                .unwrap_or(0.0),
            DensityFunction::Constant(value) => value,
            DensityFunction::YClampedGradient {
                from_y,
                to_y,
                from_value,
                to_value,
            } => {
                if block_y <= from_y {
                    from_value
                } else if block_y >= to_y {
                    to_value
                } else {
                    let progress = f64::from(block_y - from_y) / f64::from(to_y - from_y);
                    from_value + progress * (to_value - from_value)
                }
            }
            DensityFunction::Clamp { input, min, max } => input.compute(block_y).clamp(min, max),
            DensityFunction::Mapped { kind, input } => kind.transform(input.compute(block_y)),
            DensityFunction::Binary {
                kind,
                argument1,
                argument2,
            } => kind.apply(argument1.compute(block_y), argument2.compute(block_y)),
            DensityFunction::Marker { input, .. } | DensityFunction::BlendDensity { input } => {
                input.compute(block_y)
            }
            DensityFunction::BlendAlpha => 1.0,
            DensityFunction::BlendOffset => 0.0,
            DensityFunction::Noise { .. }
            | DensityFunction::ShiftedNoise { .. }
            | DensityFunction::BlendedNoise { .. }
            | DensityFunction::EndIslands { .. }
            | DensityFunction::WeirdScaledSampler { .. }
            | DensityFunction::Beardifier
            | DensityFunction::Spline
            | DensityFunction::FindTopSurface => 0.0,
        }
    }

    pub fn type_name(self) -> &'static str {
        match self {
            DensityFunction::Reference(_) => "reference",
            DensityFunction::Constant(_) => "constant",
            DensityFunction::YClampedGradient { .. } => "y_clamped_gradient",
            DensityFunction::Clamp { .. } => "clamp",
            DensityFunction::Mapped { kind, .. } => kind.serialized_name(),
            DensityFunction::Binary { kind, .. } => kind.serialized_name(),
            DensityFunction::Marker { kind, .. } => kind.serialized_name(),
            DensityFunction::Noise { .. } => "noise",
            DensityFunction::ShiftedNoise { .. } => "shifted_noise",
            DensityFunction::BlendedNoise { .. } => "old_blended_noise",
            DensityFunction::EndIslands { .. } => "end_islands",
            DensityFunction::WeirdScaledSampler { .. } => "weird_scaled_sampler",
            DensityFunction::BlendAlpha => "blend_alpha",
            DensityFunction::BlendOffset => "blend_offset",
            DensityFunction::BlendDensity { .. } => "blend_density",
            DensityFunction::Beardifier => "beardifier",
            DensityFunction::Spline => "spline",
            DensityFunction::FindTopSurface => "find_top_surface",
        }
    }
}

impl NoiseRouter {
    pub const fn simple(final_density: DensityFunction) -> Self {
        Self {
            barrier: ZERO_DENSITY,
            fluid_level_floodedness: ZERO_DENSITY,
            fluid_level_spread: ZERO_DENSITY,
            lava: ZERO_DENSITY,
            temperature: ZERO_DENSITY,
            vegetation: ZERO_DENSITY,
            continents: ZERO_DENSITY,
            erosion: ZERO_DENSITY,
            depth: ZERO_DENSITY,
            ridges: ZERO_DENSITY,
            preliminary_surface_level: ZERO_DENSITY,
            final_density,
            vein_toggle: ZERO_DENSITY,
            vein_ridged: ZERO_DENSITY,
            vein_gap: ZERO_DENSITY,
        }
    }

    pub fn field_type_names(self) -> [&'static str; 15] {
        [
            self.barrier.type_name(),
            self.fluid_level_floodedness.type_name(),
            self.fluid_level_spread.type_name(),
            self.lava.type_name(),
            self.temperature.type_name(),
            self.vegetation.type_name(),
            self.continents.type_name(),
            self.erosion.type_name(),
            self.depth.type_name(),
            self.ridges.type_name(),
            self.preliminary_surface_level.type_name(),
            self.final_density.type_name(),
            self.vein_toggle.type_name(),
            self.vein_ridged.type_name(),
            self.vein_gap.type_name(),
        ]
    }
}

impl MappedDensityFunction {
    pub fn serialized_name(self) -> &'static str {
        match self {
            MappedDensityFunction::Abs => "abs",
            MappedDensityFunction::Square => "square",
            MappedDensityFunction::Cube => "cube",
            MappedDensityFunction::HalfNegative => "half_negative",
            MappedDensityFunction::QuarterNegative => "quarter_negative",
            MappedDensityFunction::Invert => "invert",
            MappedDensityFunction::Squeeze => "squeeze",
        }
    }

    pub fn transform(self, input: f64) -> f64 {
        match self {
            MappedDensityFunction::Abs => input.abs(),
            MappedDensityFunction::Square => input * input,
            MappedDensityFunction::Cube => input * input * input,
            MappedDensityFunction::HalfNegative => {
                if input > 0.0 {
                    input
                } else {
                    input * 0.5
                }
            }
            MappedDensityFunction::QuarterNegative => {
                if input > 0.0 {
                    input
                } else {
                    input * 0.25
                }
            }
            MappedDensityFunction::Invert => -input,
            MappedDensityFunction::Squeeze => {
                let clamped = input.clamp(-1.0, 1.0);
                clamped / 2.0 - clamped * clamped * clamped / 24.0
            }
        }
    }
}

impl BinaryDensityFunction {
    pub fn serialized_name(self) -> &'static str {
        match self {
            BinaryDensityFunction::Add => "add",
            BinaryDensityFunction::Mul => "mul",
            BinaryDensityFunction::Min => "min",
            BinaryDensityFunction::Max => "max",
        }
    }

    pub fn apply(self, first: f64, second: f64) -> f64 {
        match self {
            BinaryDensityFunction::Add => first + second,
            BinaryDensityFunction::Mul => first * second,
            BinaryDensityFunction::Min => first.min(second),
            BinaryDensityFunction::Max => first.max(second),
        }
    }
}

impl DensityMarker {
    pub fn serialized_name(self) -> &'static str {
        match self {
            DensityMarker::Interpolated => "interpolated",
            DensityMarker::FlatCache => "flat_cache",
            DensityMarker::Cache2D => "cache_2d",
            DensityMarker::CacheOnce => "cache_once",
            DensityMarker::CacheAllInCell => "cache_all_in_cell",
        }
    }
}

impl RarityValueMapper {
    pub fn serialized_name(self) -> &'static str {
        match self {
            RarityValueMapper::Type1 => "type_1",
            RarityValueMapper::Type2 => "type_2",
        }
    }
}

pub fn builtin_density_function(id: &str) -> Option<&'static DensityFunctionEntry> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    BUILTIN_DENSITY_FUNCTIONS.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn density_function_type(id: &str) -> Option<&'static DensityFunctionType> {
    DENSITY_FUNCTION_TYPES.iter().find(|kind| kind.id == id)
}

pub fn builtin_noise_router(id: &str) -> Option<&'static NoiseRouterEntry> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    BUILTIN_NOISE_ROUTERS.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

#[cfg(test)]
mod tests {
    use super::{
        builtin_density_function, builtin_noise_generator_settings, builtin_noise_router,
        density_function_type, BinaryDensityFunction, DensityFunction, DensityMarker,
        MappedDensityFunction, NoiseRouterPreset, NoiseSettings, SurfaceRulePreset,
        BUILTIN_DENSITY_FUNCTIONS, BUILTIN_NOISE_GENERATOR_SETTINGS, BUILTIN_NOISE_ROUTERS,
        CAVES_NOISE_SETTINGS, DENSITY_FUNCTION_TYPES, END_NOISE_SETTINGS,
        FLOATING_ISLANDS_NOISE_SETTINGS, NETHER_NOISE_SETTINGS, OVERWORLD_NOISE_SETTINGS,
        OVERWORLD_SPAWN_TARGET, TEST_NEGATIVE_DENSITY, TEST_POSITIVE_DENSITY, Y_DENSITY,
    };
    use crate::biome::quantize_coord;

    #[test]
    fn noise_settings_presets_match_26_1_2_constants() {
        assert_eq!(OVERWORLD_NOISE_SETTINGS, NoiseSettings::new(-64, 384, 1, 2));
        assert_eq!(NETHER_NOISE_SETTINGS, NoiseSettings::new(0, 128, 1, 2));
        assert_eq!(END_NOISE_SETTINGS, NoiseSettings::new(0, 128, 2, 1));
        assert_eq!(CAVES_NOISE_SETTINGS, NoiseSettings::new(-64, 192, 1, 2));
        assert_eq!(
            FLOATING_ISLANDS_NOISE_SETTINGS,
            NoiseSettings::new(0, 256, 2, 1)
        );
        assert_eq!(OVERWORLD_NOISE_SETTINGS.cell_width(), 4);
        assert_eq!(OVERWORLD_NOISE_SETTINGS.cell_height(), 8);
    }

    #[test]
    fn noise_settings_validation_and_clamp_follow_vanilla_rules() {
        assert!(NoiseSettings::new(-64, 384, 1, 2).validate().is_ok());
        assert_eq!(
            NoiseSettings::new(-63, 384, 1, 2).validate().unwrap_err(),
            "min_y has to be a multiple of 16"
        );
        assert_eq!(
            NoiseSettings::new(-64, 383, 1, 2).validate().unwrap_err(),
            "height has to be a multiple of 16"
        );
        assert_eq!(
            NoiseSettings::new(0, 2033, 1, 2).validate().unwrap_err(),
            "min_y + height cannot be higher than: 2032"
        );
        assert_eq!(
            NoiseSettings::new(-64, 384, 1, 2).clamp_to_height(0, 255),
            NoiseSettings::new(0, 256, 1, 2)
        );
    }

    #[test]
    fn noise_generator_settings_bootstrap_matches_vanilla_order_and_flags() {
        assert_eq!(
            BUILTIN_NOISE_GENERATOR_SETTINGS
                .iter()
                .map(|settings| settings.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:overworld",
                "minecraft:large_biomes",
                "minecraft:amplified",
                "minecraft:nether",
                "minecraft:end",
                "minecraft:caves",
                "minecraft:floating_islands",
            ]
        );

        let overworld = builtin_noise_generator_settings("overworld").unwrap();
        assert_eq!(overworld.default_block, "minecraft:stone");
        assert_eq!(overworld.default_fluid, "minecraft:water");
        assert_eq!(
            overworld.noise_router,
            NoiseRouterPreset::Overworld {
                large_biomes: false,
                amplified: false
            }
        );
        assert_eq!(overworld.surface_rule, SurfaceRulePreset::Overworld);
        assert_eq!(overworld.sea_level, 63);
        assert!(overworld.aquifers_enabled);
        assert!(overworld.ore_veins_enabled);
        assert!(!overworld.legacy_random_source);

        let large = builtin_noise_generator_settings("large_biomes").unwrap();
        assert_eq!(
            large.noise_router,
            NoiseRouterPreset::Overworld {
                large_biomes: true,
                amplified: false
            }
        );
        let amplified = builtin_noise_generator_settings("amplified").unwrap();
        assert_eq!(
            amplified.noise_router,
            NoiseRouterPreset::Overworld {
                large_biomes: false,
                amplified: true
            }
        );

        let nether = builtin_noise_generator_settings("nether").unwrap();
        assert_eq!(nether.default_block, "minecraft:netherrack");
        assert_eq!(nether.default_fluid, "minecraft:lava");
        assert_eq!(nether.sea_level, 32);
        assert!(nether.legacy_random_source);

        let end = builtin_noise_generator_settings("end").unwrap();
        assert_eq!(end.default_block, "minecraft:end_stone");
        assert_eq!(end.default_fluid, "minecraft:air");
        assert!(end.disable_mob_generation);
        assert_eq!(end.sea_level, 0);
    }

    #[test]
    fn overworld_spawn_target_matches_overworld_biome_builder() {
        assert_eq!(OVERWORLD_SPAWN_TARGET.len(), 2);
        assert_eq!(
            OVERWORLD_SPAWN_TARGET[0].continentalness.min,
            quantize_coord(-0.11)
        );
        assert_eq!(
            OVERWORLD_SPAWN_TARGET[0].continentalness.max,
            quantize_coord(1.0)
        );
        assert_eq!(
            OVERWORLD_SPAWN_TARGET[0].weirdness.min,
            quantize_coord(-1.0)
        );
        assert_eq!(
            OVERWORLD_SPAWN_TARGET[0].weirdness.max,
            quantize_coord(-0.16)
        );
        assert_eq!(
            OVERWORLD_SPAWN_TARGET[1].weirdness.min,
            quantize_coord(0.16)
        );
        assert_eq!(OVERWORLD_SPAWN_TARGET[1].weirdness.max, quantize_coord(1.0));
    }

    #[test]
    fn density_function_type_registry_matches_densityfunctions_bootstrap_order() {
        assert_eq!(DENSITY_FUNCTION_TYPES.len(), 34);
        assert_eq!(
            DENSITY_FUNCTION_TYPES
                .iter()
                .map(|kind| kind.id)
                .collect::<Vec<_>>(),
            vec![
                "blend_alpha",
                "blend_offset",
                "beardifier",
                "old_blended_noise",
                "interpolated",
                "flat_cache",
                "cache_2d",
                "cache_once",
                "cache_all_in_cell",
                "noise",
                "end_islands",
                "weird_scaled_sampler",
                "shifted_noise",
                "range_choice",
                "shift_a",
                "shift_b",
                "shift",
                "blend_density",
                "clamp",
                "abs",
                "square",
                "cube",
                "half_negative",
                "quarter_negative",
                "invert",
                "squeeze",
                "add",
                "mul",
                "min",
                "max",
                "spline",
                "constant",
                "y_clamped_gradient",
                "find_top_surface",
            ]
        );
        assert!(density_function_type("shifted_noise").is_some());
        assert!(density_function_type("missing").is_none());
    }

    #[test]
    fn density_function_core_evaluators_follow_vanilla_transform_rules() {
        assert_eq!(Y_DENSITY.compute(-5000), -4064.0);
        assert_eq!(Y_DENSITY.compute(5000), 4062.0);
        assert_eq!(Y_DENSITY.compute(0), 0.0);

        assert_eq!(MappedDensityFunction::Abs.transform(-2.0), 2.0);
        assert_eq!(MappedDensityFunction::Square.transform(-2.0), 4.0);
        assert_eq!(MappedDensityFunction::Cube.transform(-2.0), -8.0);
        assert_eq!(MappedDensityFunction::HalfNegative.transform(-2.0), -1.0);
        assert_eq!(MappedDensityFunction::QuarterNegative.transform(-2.0), -0.5);
        assert_eq!(MappedDensityFunction::Invert.transform(2.0), -2.0);
        assert!((MappedDensityFunction::Squeeze.transform(1.0) - 0.4583333333333333).abs() < 1e-12);

        let add = DensityFunction::Binary {
            kind: BinaryDensityFunction::Add,
            argument1: &TEST_NEGATIVE_DENSITY,
            argument2: &TEST_POSITIVE_DENSITY,
        };
        assert_eq!(add.compute(0), 1.0);
        assert_eq!(BinaryDensityFunction::Mul.apply(-2.0, 3.0), -6.0);
        assert_eq!(BinaryDensityFunction::Min.apply(-2.0, 3.0), -2.0);
        assert_eq!(BinaryDensityFunction::Max.apply(-2.0, 3.0), 3.0);
    }

    #[test]
    fn noise_router_density_function_bootstrap_keys_match_vanilla_prefix() {
        assert_eq!(
            BUILTIN_DENSITY_FUNCTIONS
                .iter()
                .map(|entry| entry.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:zero",
                "minecraft:y",
                "minecraft:shift_x",
                "minecraft:shift_z",
                "minecraft:overworld/base_3d_noise",
                "minecraft:nether/base_3d_noise",
                "minecraft:end/base_3d_noise",
                "minecraft:overworld/continents",
                "minecraft:overworld/erosion",
                "minecraft:overworld/ridges",
                "minecraft:overworld/ridges_folded",
                "minecraft:overworld_large_biomes/continents",
                "minecraft:overworld_large_biomes/erosion",
                "minecraft:end/sloped_cheese",
                "minecraft:overworld/caves/spaghetti_2d_thickness_modulator",
            ]
        );
        assert_eq!(
            builtin_density_function("overworld/base_3d_noise")
                .unwrap()
                .function
                .type_name(),
            "old_blended_noise"
        );
        assert_eq!(
            builtin_density_function("shift_x")
                .unwrap()
                .function
                .type_name(),
            DensityMarker::FlatCache.serialized_name()
        );
        assert_eq!(
            builtin_density_function("overworld/caves/spaghetti_2d_thickness_modulator")
                .unwrap()
                .function
                .type_name(),
            "cache_once"
        );
    }

    #[test]
    fn noise_router_record_shape_and_presets_match_noise_router_data() {
        assert_eq!(
            BUILTIN_NOISE_ROUTERS
                .iter()
                .map(|entry| entry.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:overworld",
                "minecraft:large_biomes",
                "minecraft:amplified",
                "minecraft:nether",
                "minecraft:end",
                "minecraft:caves",
                "minecraft:floating_islands",
                "minecraft:none",
            ]
        );

        let overworld = builtin_noise_router("overworld").unwrap().router;
        assert_eq!(
            overworld.field_type_names(),
            [
                "noise",
                "noise",
                "noise",
                "noise",
                "shifted_noise",
                "shifted_noise",
                "reference",
                "reference",
                "reference",
                "reference",
                "reference",
                "reference",
                "noise",
                "reference",
                "noise",
            ]
        );
        assert_eq!(
            overworld.barrier,
            DensityFunction::Noise {
                noise: "minecraft:aquifer_barrier",
                xz_scale: 1.0,
                y_scale: 0.5,
            }
        );
        assert_eq!(
            overworld.final_density,
            DensityFunction::Reference("minecraft:overworld/final_density")
        );

        let large = builtin_noise_router("large_biomes").unwrap().router;
        assert_eq!(
            large.temperature,
            DensityFunction::ShiftedNoise {
                shift_x: &super::SHIFT_X_DENSITY,
                shift_y: &super::ZERO_DENSITY,
                shift_z: &super::SHIFT_Z_DENSITY,
                xz_scale: 0.25,
                y_scale: 0.0,
                noise: "minecraft:temperature_large",
            }
        );
        assert_eq!(
            large.continents,
            DensityFunction::Reference("minecraft:overworld_large_biomes/continents")
        );

        let nether = builtin_noise_router("nether").unwrap().router;
        assert_eq!(
            nether.temperature,
            DensityFunction::ShiftedNoise {
                shift_x: &super::ZERO_DENSITY,
                shift_y: &super::ZERO_DENSITY,
                shift_z: &super::ZERO_DENSITY,
                xz_scale: 0.25,
                y_scale: 0.0,
                noise: "minecraft:temperature_nether",
            }
        );
        assert_eq!(
            builtin_noise_router("end")
                .unwrap()
                .router
                .erosion
                .type_name(),
            "cache_2d"
        );
        assert_eq!(
            builtin_noise_router("none").unwrap().router.final_density,
            DensityFunction::Constant(0.0)
        );
    }
}
