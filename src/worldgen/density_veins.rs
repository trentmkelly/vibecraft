use super::*;

// === Vein density functions ===
// Mirrors Java NoiseRouterData.overworld() vein computation (same for all overworld variants).
// Java: yLimitedInterpolatable(y, noise(ORE_VEININESS, 1.5, 1.5), veinMinY, veinMaxY, 0)
//   where veinMinY = min(IRON.minY, COPPER.minY) = min(-60, 0) = -60
//         veinMaxY = max(IRON.maxY, COPPER.maxY) = max(-8, 50) = 50
// yLimitedInterpolatable(y, inner, min, max, out) = Interpolated(rangeChoice(y, min, max+1, inner, out))

pub const OVERWORLD_VEIN_TOGGLE_NOISE_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:ore_veininess",
    xz_scale: 1.5,
    y_scale: 1.5,
};
pub const OVERWORLD_VEIN_TOGGLE_RANGE_DENSITY: DensityFunction = DensityFunction::RangeChoice {
    input: &Y_DENSITY,
    min_inclusive: -60.0,
    max_exclusive: 51.0, // veinMaxY + 1 = 50 + 1
    when_in_range: &OVERWORLD_VEIN_TOGGLE_NOISE_DENSITY,
    when_out_of_range: &ZERO_DENSITY,
};
pub const OVERWORLD_VEIN_TOGGLE_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Interpolated,
    input: &OVERWORLD_VEIN_TOGGLE_RANGE_DENSITY,
};

// Java: veinA = yLimitedInterpolatable(y, noise(ORE_VEIN_A, 4.0, 4.0), veinMinY, veinMaxY, 0).abs()
pub const OVERWORLD_VEIN_A_NOISE_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:ore_vein_a",
    xz_scale: 4.0,
    y_scale: 4.0,
};
pub const OVERWORLD_VEIN_A_RANGE_DENSITY: DensityFunction = DensityFunction::RangeChoice {
    input: &Y_DENSITY,
    min_inclusive: -60.0,
    max_exclusive: 51.0,
    when_in_range: &OVERWORLD_VEIN_A_NOISE_DENSITY,
    when_out_of_range: &ZERO_DENSITY,
};
pub const OVERWORLD_VEIN_A_INTERPOLATED_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Interpolated,
    input: &OVERWORLD_VEIN_A_RANGE_DENSITY,
};
pub const OVERWORLD_VEIN_A_DENSITY: DensityFunction = DensityFunction::Mapped {
    kind: MappedDensityFunction::Abs,
    input: &OVERWORLD_VEIN_A_INTERPOLATED_DENSITY,
};

// Java: veinB = yLimitedInterpolatable(y, noise(ORE_VEIN_B, 4.0, 4.0), veinMinY, veinMaxY, 0).abs()
pub const OVERWORLD_VEIN_B_NOISE_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:ore_vein_b",
    xz_scale: 4.0,
    y_scale: 4.0,
};
pub const OVERWORLD_VEIN_B_RANGE_DENSITY: DensityFunction = DensityFunction::RangeChoice {
    input: &Y_DENSITY,
    min_inclusive: -60.0,
    max_exclusive: 51.0,
    when_in_range: &OVERWORLD_VEIN_B_NOISE_DENSITY,
    when_out_of_range: &ZERO_DENSITY,
};
pub const OVERWORLD_VEIN_B_INTERPOLATED_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Interpolated,
    input: &OVERWORLD_VEIN_B_RANGE_DENSITY,
};
pub const OVERWORLD_VEIN_B_DENSITY: DensityFunction = DensityFunction::Mapped {
    kind: MappedDensityFunction::Abs,
    input: &OVERWORLD_VEIN_B_INTERPOLATED_DENSITY,
};

// Java: veinRidged = add(constant(-0.08F), max(veinA, veinB))
// -0.08F widened to double = -0.07999999821186066
pub const OVERWORLD_VEIN_MAX_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Max,
    argument1: &OVERWORLD_VEIN_A_DENSITY,
    argument2: &OVERWORLD_VEIN_B_DENSITY,
};
pub const OVERWORLD_VEIN_RIDGED_OFFSET_DENSITY: DensityFunction =
    DensityFunction::Constant(-0.07999999821186066);
pub const OVERWORLD_VEIN_RIDGED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &OVERWORLD_VEIN_RIDGED_OFFSET_DENSITY,
    argument2: &OVERWORLD_VEIN_MAX_DENSITY,
};

