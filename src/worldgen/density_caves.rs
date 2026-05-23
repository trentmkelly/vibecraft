use super::*;

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
pub const SPAGHETTI_ROUGHNESS_MODULATOR_NOISE_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:spaghetti_roughness_modulator",
    xz_scale: 1.0,
    y_scale: 1.0,
};
pub const SPAGHETTI_ROUGHNESS_MODULATOR_SCALE_DENSITY: DensityFunction =
    DensityFunction::Constant(-0.05);
pub const SPAGHETTI_ROUGHNESS_MODULATED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &SPAGHETTI_ROUGHNESS_MODULATOR_SCALE_DENSITY,
    argument2: &SPAGHETTI_ROUGHNESS_MODULATOR_NOISE_DENSITY,
};
pub const SPAGHETTI_ROUGHNESS_MODULATOR_OFFSET_DENSITY: DensityFunction =
    DensityFunction::Constant(-0.05);
pub const SPAGHETTI_ROUGHNESS_FIRST_FACTOR_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &SPAGHETTI_ROUGHNESS_MODULATOR_OFFSET_DENSITY,
    argument2: &SPAGHETTI_ROUGHNESS_MODULATED_DENSITY,
};
pub const SPAGHETTI_ROUGHNESS_NOISE_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:spaghetti_roughness",
    xz_scale: 1.0,
    y_scale: 1.0,
};
pub const SPAGHETTI_ROUGHNESS_ABS_DENSITY: DensityFunction = DensityFunction::Mapped {
    kind: MappedDensityFunction::Abs,
    input: &SPAGHETTI_ROUGHNESS_NOISE_DENSITY,
};
pub const SPAGHETTI_ROUGHNESS_SECOND_OFFSET_DENSITY: DensityFunction =
    DensityFunction::Constant(-0.4);
pub const SPAGHETTI_ROUGHNESS_SECOND_FACTOR_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &SPAGHETTI_ROUGHNESS_SECOND_OFFSET_DENSITY,
    argument2: &SPAGHETTI_ROUGHNESS_ABS_DENSITY,
};
pub const SPAGHETTI_ROUGHNESS_FUNCTION_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &SPAGHETTI_ROUGHNESS_FIRST_FACTOR_DENSITY,
    argument2: &SPAGHETTI_ROUGHNESS_SECOND_FACTOR_DENSITY,
};
pub const PILLAR_NOISE_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:pillar",
    xz_scale: 25.0,
    y_scale: 0.3,
};
pub const PILLAR_SCALE_DENSITY: DensityFunction = DensityFunction::Constant(2.0);
pub const PILLAR_SCALED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &PILLAR_SCALE_DENSITY,
    argument2: &PILLAR_NOISE_DENSITY,
};
pub const PILLAR_RARENESS_NOISE_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:pillar_rareness",
    xz_scale: 1.0,
    y_scale: 1.0,
};
pub const PILLAR_RARENESS_INVERT_SCALE_DENSITY: DensityFunction = DensityFunction::Constant(-1.0);
pub const PILLAR_RARENESS_INVERTED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &PILLAR_RARENESS_INVERT_SCALE_DENSITY,
    argument2: &PILLAR_RARENESS_NOISE_DENSITY,
};
pub const PILLAR_RARENESS_OFFSET_DENSITY: DensityFunction = DensityFunction::Constant(-1.0);
pub const PILLAR_RARENESS_FACTOR_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PILLAR_RARENESS_OFFSET_DENSITY,
    argument2: &PILLAR_RARENESS_INVERTED_DENSITY,
};
pub const PILLAR_FIRST_FACTOR_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PILLAR_SCALED_DENSITY,
    argument2: &PILLAR_RARENESS_FACTOR_DENSITY,
};
pub const PILLAR_THICKNESS_NOISE_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:pillar_thickness",
    xz_scale: 1.0,
    y_scale: 1.0,
};
pub const PILLAR_THICKNESS_SCALE_DENSITY: DensityFunction = DensityFunction::Constant(0.55);
pub const PILLAR_THICKNESS_SCALED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &PILLAR_THICKNESS_SCALE_DENSITY,
    argument2: &PILLAR_THICKNESS_NOISE_DENSITY,
};
pub const PILLAR_THICKNESS_OFFSET_DENSITY: DensityFunction = DensityFunction::Constant(0.55);
pub const PILLAR_THICKNESS_FACTOR_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PILLAR_THICKNESS_OFFSET_DENSITY,
    argument2: &PILLAR_THICKNESS_SCALED_DENSITY,
};
pub const PILLAR_THICKNESS_CUBED_DENSITY: DensityFunction = DensityFunction::Mapped {
    kind: MappedDensityFunction::Cube,
    input: &PILLAR_THICKNESS_FACTOR_DENSITY,
};
pub const PILLARS_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &PILLAR_FIRST_FACTOR_DENSITY,
    argument2: &PILLAR_THICKNESS_CUBED_DENSITY,
};
pub const SPAGHETTI_2D_MODULATOR_NOISE_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:spaghetti_2d_modulator",
    xz_scale: 2.0,
    y_scale: 1.0,
};
pub const SPAGHETTI_2D_WEIRD_SCALED_DENSITY: DensityFunction =
    DensityFunction::WeirdScaledSampler {
        input: &SPAGHETTI_2D_MODULATOR_NOISE_DENSITY,
        noise: "minecraft:spaghetti_2d",
        rarity_mapper: RarityValueMapper::Type2,
    };
pub const SPAGHETTI_2D_THICKNESS_WEIGHT_DENSITY: DensityFunction = DensityFunction::Constant(0.083);
pub const SPAGHETTI_2D_THICKNESS_WEIGHTED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &SPAGHETTI_2D_THICKNESS_WEIGHT_DENSITY,
    argument2: &SPAGHETTI_2D_THICKNESS_MODULATOR_REFERENCE_DENSITY,
};
pub const SPAGHETTI_2D_THICKNESS_MODULATOR_REFERENCE_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld/caves/spaghetti_2d_thickness_modulator");
pub const SPAGHETTI_2D_FIRST_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &SPAGHETTI_2D_WEIRD_SCALED_DENSITY,
    argument2: &SPAGHETTI_2D_THICKNESS_WEIGHTED_DENSITY,
};
pub const SPAGHETTI_2D_ELEVATION_NOISE_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:spaghetti_2d_elevation",
    xz_scale: 1.0,
    y_scale: 0.0,
};
pub const SPAGHETTI_2D_ELEVATION_SCALE_DENSITY: DensityFunction = DensityFunction::Constant(8.0);
pub const SPAGHETTI_2D_ELEVATION_SCALED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &SPAGHETTI_2D_ELEVATION_SCALE_DENSITY,
    argument2: &SPAGHETTI_2D_ELEVATION_NOISE_DENSITY,
};
pub const SPAGHETTI_2D_ELEVATION_ZERO_DENSITY: DensityFunction = DensityFunction::Constant(0.0);
pub const SPAGHETTI_2D_ELEVATION_OFFSET_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &SPAGHETTI_2D_ELEVATION_ZERO_DENSITY,
    argument2: &SPAGHETTI_2D_ELEVATION_SCALED_DENSITY,
};
pub const SPAGHETTI_2D_Y_GRADIENT_DENSITY: DensityFunction = DensityFunction::YClampedGradient {
    from_y: -64,
    to_y: 320,
    from_value: 8.0,
    to_value: -40.0,
};
pub const SPAGHETTI_2D_ELEVATION_WITH_GRADIENT_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &SPAGHETTI_2D_ELEVATION_OFFSET_DENSITY,
    argument2: &SPAGHETTI_2D_Y_GRADIENT_DENSITY,
};
pub const SPAGHETTI_2D_ELEVATION_ABS_DENSITY: DensityFunction = DensityFunction::Mapped {
    kind: MappedDensityFunction::Abs,
    input: &SPAGHETTI_2D_ELEVATION_WITH_GRADIENT_DENSITY,
};
pub const SPAGHETTI_2D_SECOND_INPUT_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &SPAGHETTI_2D_ELEVATION_ABS_DENSITY,
    argument2: &SPAGHETTI_2D_THICKNESS_MODULATOR_REFERENCE_DENSITY,
};
pub const SPAGHETTI_2D_SECOND_DENSITY: DensityFunction = DensityFunction::Mapped {
    kind: MappedDensityFunction::Cube,
    input: &SPAGHETTI_2D_SECOND_INPUT_DENSITY,
};
pub const SPAGHETTI_2D_MAX_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Max,
    argument1: &SPAGHETTI_2D_FIRST_DENSITY,
    argument2: &SPAGHETTI_2D_SECOND_DENSITY,
};
pub const SPAGHETTI_2D_DENSITY: DensityFunction = DensityFunction::Clamp {
    input: &SPAGHETTI_2D_MAX_DENSITY,
    min: -1.0,
    max: 1.0,
};
pub const NOODLE_Y_MIN_DENSITY: f64 = -60.0;
pub const NOODLE_Y_MAX_DENSITY: f64 = 321.0;
pub const NOODLE_NOISE_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:noodle",
    xz_scale: 1.0,
    y_scale: 1.0,
};
pub const NOODLE_OUT_OF_RANGE_DENSITY: DensityFunction = DensityFunction::Constant(-1.0);
pub const NOODLE_TOGGLE_RANGE_DENSITY: DensityFunction = DensityFunction::RangeChoice {
    input: &Y_DENSITY,
    min_inclusive: NOODLE_Y_MIN_DENSITY,
    max_exclusive: NOODLE_Y_MAX_DENSITY,
    when_in_range: &NOODLE_NOISE_DENSITY,
    when_out_of_range: &NOODLE_OUT_OF_RANGE_DENSITY,
};
pub const NOODLE_TOGGLE_INTERPOLATED_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Interpolated,
    input: &NOODLE_TOGGLE_RANGE_DENSITY,
};
pub const NOODLE_BLOCKING_DENSITY: DensityFunction = DensityFunction::Constant(64.0);
pub const NOODLE_THICKNESS_NOISE_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:noodle_thickness",
    xz_scale: 1.0,
    y_scale: 1.0,
};
pub const NOODLE_THICKNESS_SCALE_DENSITY: DensityFunction = DensityFunction::Constant(-0.025);
pub const NOODLE_THICKNESS_SCALED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &NOODLE_THICKNESS_SCALE_DENSITY,
    argument2: &NOODLE_THICKNESS_NOISE_DENSITY,
};
pub const NOODLE_THICKNESS_OFFSET_DENSITY: DensityFunction =
    DensityFunction::Constant(-0.07500000000000001);
pub const NOODLE_THICKNESS_IN_RANGE_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &NOODLE_THICKNESS_OFFSET_DENSITY,
    argument2: &NOODLE_THICKNESS_SCALED_DENSITY,
};
pub const NOODLE_ZERO_DENSITY: DensityFunction = DensityFunction::Constant(0.0);
pub const NOODLE_THICKNESS_RANGE_DENSITY: DensityFunction = DensityFunction::RangeChoice {
    input: &Y_DENSITY,
    min_inclusive: NOODLE_Y_MIN_DENSITY,
    max_exclusive: NOODLE_Y_MAX_DENSITY,
    when_in_range: &NOODLE_THICKNESS_IN_RANGE_DENSITY,
    when_out_of_range: &NOODLE_ZERO_DENSITY,
};
pub const NOODLE_THICKNESS_INTERPOLATED_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Interpolated,
    input: &NOODLE_THICKNESS_RANGE_DENSITY,
};
pub const NOODLE_RIDGE_A_NOISE_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:noodle_ridge_a",
    xz_scale: 2.6666666666666665,
    y_scale: 2.6666666666666665,
};
pub const NOODLE_RIDGE_A_RANGE_DENSITY: DensityFunction = DensityFunction::RangeChoice {
    input: &Y_DENSITY,
    min_inclusive: NOODLE_Y_MIN_DENSITY,
    max_exclusive: NOODLE_Y_MAX_DENSITY,
    when_in_range: &NOODLE_RIDGE_A_NOISE_DENSITY,
    when_out_of_range: &NOODLE_ZERO_DENSITY,
};
pub const NOODLE_RIDGE_A_INTERPOLATED_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Interpolated,
    input: &NOODLE_RIDGE_A_RANGE_DENSITY,
};
pub const NOODLE_RIDGE_A_ABS_DENSITY: DensityFunction = DensityFunction::Mapped {
    kind: MappedDensityFunction::Abs,
    input: &NOODLE_RIDGE_A_INTERPOLATED_DENSITY,
};
pub const NOODLE_RIDGE_B_NOISE_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:noodle_ridge_b",
    xz_scale: 2.6666666666666665,
    y_scale: 2.6666666666666665,
};
pub const NOODLE_RIDGE_B_RANGE_DENSITY: DensityFunction = DensityFunction::RangeChoice {
    input: &Y_DENSITY,
    min_inclusive: NOODLE_Y_MIN_DENSITY,
    max_exclusive: NOODLE_Y_MAX_DENSITY,
    when_in_range: &NOODLE_RIDGE_B_NOISE_DENSITY,
    when_out_of_range: &NOODLE_ZERO_DENSITY,
};
pub const NOODLE_RIDGE_B_INTERPOLATED_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Interpolated,
    input: &NOODLE_RIDGE_B_RANGE_DENSITY,
};
pub const NOODLE_RIDGE_B_ABS_DENSITY: DensityFunction = DensityFunction::Mapped {
    kind: MappedDensityFunction::Abs,
    input: &NOODLE_RIDGE_B_INTERPOLATED_DENSITY,
};
pub const NOODLE_RIDGE_MAX_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Max,
    argument1: &NOODLE_RIDGE_A_ABS_DENSITY,
    argument2: &NOODLE_RIDGE_B_ABS_DENSITY,
};
pub const NOODLE_RIDGE_SCALE_DENSITY: DensityFunction = DensityFunction::Constant(1.5);
pub const NOODLE_RIDGE_SCALED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &NOODLE_RIDGE_SCALE_DENSITY,
    argument2: &NOODLE_RIDGE_MAX_DENSITY,
};
pub const NOODLE_OPEN_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &NOODLE_THICKNESS_INTERPOLATED_DENSITY,
    argument2: &NOODLE_RIDGE_SCALED_DENSITY,
};
pub const NOODLE_DENSITY: DensityFunction = DensityFunction::RangeChoice {
    input: &NOODLE_TOGGLE_INTERPOLATED_DENSITY,
    min_inclusive: -1000000.0,
    max_exclusive: 0.0,
    when_in_range: &NOODLE_BLOCKING_DENSITY,
    when_out_of_range: &NOODLE_OPEN_DENSITY,
};
pub const ENTRANCES_CAVE_NOISE_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:cave_entrance",
    xz_scale: 0.75,
    y_scale: 0.5,
};
pub const ENTRANCES_CAVE_OFFSET_DENSITY: DensityFunction = DensityFunction::Constant(0.37);
pub const ENTRANCES_CAVE_OFFSET_NOISE_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &ENTRANCES_CAVE_OFFSET_DENSITY,
    argument2: &ENTRANCES_CAVE_NOISE_DENSITY,
};
pub const ENTRANCES_CAVE_GRADIENT_DENSITY: DensityFunction = DensityFunction::YClampedGradient {
    from_y: -10,
    to_y: 30,
    from_value: 0.3,
    to_value: 0.0,
};
pub const ENTRANCES_CAVE_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &ENTRANCES_CAVE_OFFSET_NOISE_DENSITY,
    argument2: &ENTRANCES_CAVE_GRADIENT_DENSITY,
};
pub const ENTRANCES_ROUGHNESS_REFERENCE_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld/caves/spaghetti_roughness_function");
pub const ENTRANCES_RARITY_NOISE_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:spaghetti_3d_rarity",
    xz_scale: 2.0,
    y_scale: 1.0,
};
pub const ENTRANCES_RARITY_CACHE_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::CacheOnce,
    input: &ENTRANCES_RARITY_NOISE_DENSITY,
};
pub const ENTRANCES_SPAGHETTI_3D_1_DENSITY: DensityFunction = DensityFunction::WeirdScaledSampler {
    input: &ENTRANCES_RARITY_CACHE_DENSITY,
    noise: "minecraft:spaghetti_3d_1",
    rarity_mapper: RarityValueMapper::Type1,
};
pub const ENTRANCES_SPAGHETTI_3D_2_DENSITY: DensityFunction = DensityFunction::WeirdScaledSampler {
    input: &ENTRANCES_RARITY_CACHE_DENSITY,
    noise: "minecraft:spaghetti_3d_2",
    rarity_mapper: RarityValueMapper::Type1,
};
pub const ENTRANCES_SPAGHETTI_MAX_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Max,
    argument1: &ENTRANCES_SPAGHETTI_3D_1_DENSITY,
    argument2: &ENTRANCES_SPAGHETTI_3D_2_DENSITY,
};
pub const ENTRANCES_THICKNESS_NOISE_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:spaghetti_3d_thickness",
    xz_scale: 1.0,
    y_scale: 1.0,
};
pub const ENTRANCES_THICKNESS_SCALE_DENSITY: DensityFunction =
    DensityFunction::Constant(-0.011499999999999996);
pub const ENTRANCES_THICKNESS_SCALED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &ENTRANCES_THICKNESS_SCALE_DENSITY,
    argument2: &ENTRANCES_THICKNESS_NOISE_DENSITY,
};
pub const ENTRANCES_THICKNESS_OFFSET_DENSITY: DensityFunction = DensityFunction::Constant(-0.0765);
pub const ENTRANCES_THICKNESS_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &ENTRANCES_THICKNESS_OFFSET_DENSITY,
    argument2: &ENTRANCES_THICKNESS_SCALED_DENSITY,
};
pub const ENTRANCES_SPAGHETTI_WITH_THICKNESS_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &ENTRANCES_SPAGHETTI_MAX_DENSITY,
    argument2: &ENTRANCES_THICKNESS_DENSITY,
};
pub const ENTRANCES_SPAGHETTI_CLAMPED_DENSITY: DensityFunction = DensityFunction::Clamp {
    input: &ENTRANCES_SPAGHETTI_WITH_THICKNESS_DENSITY,
    min: -1.0,
    max: 1.0,
};
pub const ENTRANCES_SPAGHETTI_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &ENTRANCES_ROUGHNESS_REFERENCE_DENSITY,
    argument2: &ENTRANCES_SPAGHETTI_CLAMPED_DENSITY,
};
pub const ENTRANCES_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Min,
    argument1: &ENTRANCES_CAVE_DENSITY,
    argument2: &ENTRANCES_SPAGHETTI_DENSITY,
};
