use super::*;

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
pub const SHIFT_A_DENSITY: DensityFunction = DensityFunction::ShiftA {
    noise: "minecraft:offset",
};
pub const SHIFT_Z_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::FlatCache,
    input: &SHIFT_Z_CACHE_2D_DENSITY,
};
pub const SHIFT_Z_CACHE_2D_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Cache2D,
    input: &SHIFT_B_DENSITY,
};
pub const SHIFT_B_DENSITY: DensityFunction = DensityFunction::ShiftB {
    noise: "minecraft:offset",
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
        function: RIDGE_FOLDED_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/offset",
        function: OVERWORLD_OFFSET_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/factor",
        function: OVERWORLD_FACTOR_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/jaggedness",
        function: OVERWORLD_JAGGEDNESS_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/depth",
        function: OVERWORLD_DEPTH_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/sloped_cheese",
        function: OVERWORLD_SLOPED_CHEESE_DENSITY,
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
        id: "minecraft:overworld_large_biomes/offset",
        function: OVERWORLD_LARGE_BIOMES_OFFSET_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:overworld_large_biomes/factor",
        function: OVERWORLD_LARGE_BIOMES_FACTOR_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:overworld_large_biomes/jaggedness",
        function: OVERWORLD_LARGE_BIOMES_JAGGEDNESS_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:overworld_large_biomes/depth",
        function: OVERWORLD_LARGE_BIOMES_DEPTH_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:overworld_large_biomes/sloped_cheese",
        function: OVERWORLD_LARGE_BIOMES_SLOPED_CHEESE_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:overworld_amplified/offset",
        function: OVERWORLD_AMPLIFIED_OFFSET_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:overworld_amplified/factor",
        function: OVERWORLD_AMPLIFIED_FACTOR_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:overworld_amplified/jaggedness",
        function: OVERWORLD_AMPLIFIED_JAGGEDNESS_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:overworld_amplified/depth",
        function: OVERWORLD_AMPLIFIED_DEPTH_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:overworld_amplified/sloped_cheese",
        function: OVERWORLD_AMPLIFIED_SLOPED_CHEESE_DENSITY,
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
    DensityFunctionEntry {
        id: "minecraft:overworld/caves/spaghetti_roughness_function",
        function: DensityFunction::Marker {
            kind: DensityMarker::CacheOnce,
            input: &SPAGHETTI_ROUGHNESS_FUNCTION_DENSITY,
        },
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/caves/pillars",
        function: DensityFunction::Marker {
            kind: DensityMarker::CacheOnce,
            input: &PILLARS_DENSITY,
        },
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/caves/spaghetti_2d",
        function: SPAGHETTI_2D_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/caves/noodle",
        function: NOODLE_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/caves/entrances",
        function: DensityFunction::Marker {
            kind: DensityMarker::CacheOnce,
            input: &ENTRANCES_DENSITY,
        },
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/final_density",
        function: OVERWORLD_FINAL_DENSITY_CONST,
    },
];
