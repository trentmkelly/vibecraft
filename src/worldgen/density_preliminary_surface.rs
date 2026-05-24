use super::*;

// === Preliminary surface level density functions ===
// Mirrors Java NoiseRouterData.preliminarySurfaceLevel(offset, factor, amplified).
// Java source: NoiseRouterData.java lines 479-501.
//
// The function uses cache2d-wrapped offset and factor references, builds an upper bound via
// remap(), then runs a simplified slideOverworld() density to feed findTopSurface().
//
// remap(input, 1.5, -1.5, -64.0, 320.0) expands to add(mul(input, -128.0), 128.0):
//   factor = (320 - (-64)) / (-1.5 - 1.5) = -128.0
//   offset = -64 - 1.5 * (-128) = 128.0
//
// noiseGradientDensity(f, d) = mul(constant(4.0), quarterNegative(mul(d, f)))
// offsetToDepth(offset) = add(yClampedGradient(-64, 320, 1.5, -1.5), offset)
//
// findTopSurface lower_bound=-64, cell_height=NoiseSettings.OVERWORLD.getCellHeight()=8
//   (noiseSizeVertical=2, getCellHeight()=QuartPos.toBlock(2)=8)

// --- Standard overworld preliminary_surface_level (amplified=false) ---
// Uses Reference("minecraft:overworld/factor") and Reference("minecraft:overworld/offset").

pub const PRELIM_OW_FACTOR_REF_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld/factor");
pub const PRELIM_OW_OFFSET_REF_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld/offset");

pub const PRELIM_OW_CACHED_FACTOR_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Cache2D,
    input: &PRELIM_OW_FACTOR_REF_DENSITY,
};
pub const PRELIM_OW_CACHED_OFFSET_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Cache2D,
    input: &PRELIM_OW_OFFSET_REF_DENSITY,
};

// Upper bound computation:
// add(mul(constant(0.2734375), cachedFactor.invert()), mul(constant(-1.0), cachedOffset))
pub const PRELIM_OW_FACTOR_INVERT_SCALE_DENSITY: DensityFunction =
    DensityFunction::Constant(0.2734375);
pub const PRELIM_OW_FACTOR_INVERTED_DENSITY: DensityFunction = DensityFunction::Mapped {
    kind: MappedDensityFunction::Invert,
    input: &PRELIM_OW_CACHED_FACTOR_DENSITY,
};
pub const PRELIM_OW_SCALED_INVERT_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &PRELIM_OW_FACTOR_INVERT_SCALE_DENSITY,
    argument2: &PRELIM_OW_FACTOR_INVERTED_DENSITY,
};
pub const PRELIM_OW_NEG_OFFSET_SCALE_DENSITY: DensityFunction = DensityFunction::Constant(-1.0);
pub const PRELIM_OW_NEG_CACHED_OFFSET_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &PRELIM_OW_NEG_OFFSET_SCALE_DENSITY,
    argument2: &PRELIM_OW_CACHED_OFFSET_DENSITY,
};
pub const PRELIM_OW_UPPER_BOUND_SUM_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_OW_SCALED_INVERT_DENSITY,
    argument2: &PRELIM_OW_NEG_CACHED_OFFSET_DENSITY,
};
// remap: add(mul(sum, -128.0), 128.0)
pub const PRELIM_REMAP_SCALE_DENSITY: DensityFunction = DensityFunction::Constant(-128.0);
pub const PRELIM_REMAP_OFFSET_DENSITY: DensityFunction = DensityFunction::Constant(128.0);
pub const PRELIM_OW_UPPER_BOUND_SCALED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &PRELIM_OW_UPPER_BOUND_SUM_DENSITY,
    argument2: &PRELIM_REMAP_SCALE_DENSITY,
};
pub const PRELIM_OW_UPPER_BOUND_REMAPPED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_OW_UPPER_BOUND_SCALED_DENSITY,
    argument2: &PRELIM_REMAP_OFFSET_DENSITY,
};
// clamp(-40.0, 320.0)
pub const PRELIM_OW_UPPER_BOUND_DENSITY: DensityFunction = DensityFunction::Clamp {
    input: &PRELIM_OW_UPPER_BOUND_REMAPPED_DENSITY,
    min: -40.0,
    max: 320.0,
};

// Density computation:
// offsetToDepth(cachedOffset) = add(yClampedGradient(-64, 320, 1.5, -1.5), cachedOffset)
pub const PRELIM_DEPTH_GRADIENT_DENSITY: DensityFunction = DensityFunction::YClampedGradient {
    from_y: -64,
    to_y: 320,
    from_value: 1.5,
    to_value: -1.5,
};
pub const PRELIM_OW_OFFSET_TO_DEPTH_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_DEPTH_GRADIENT_DENSITY,
    argument2: &PRELIM_OW_CACHED_OFFSET_DENSITY,
};
// noiseGradientDensity(cachedFactor, offsetToDepth) = mul(4.0, quarterNegative(mul(depth, factor)))
pub const PRELIM_OW_DEPTH_MUL_FACTOR_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &PRELIM_OW_OFFSET_TO_DEPTH_DENSITY,
    argument2: &PRELIM_OW_CACHED_FACTOR_DENSITY,
};
pub const PRELIM_OW_QUARTER_NEG_DENSITY: DensityFunction = DensityFunction::Mapped {
    kind: MappedDensityFunction::QuarterNegative,
    input: &PRELIM_OW_DEPTH_MUL_FACTOR_DENSITY,
};
pub const PRELIM_SCALE_4_DENSITY: DensityFunction = DensityFunction::Constant(4.0);
pub const PRELIM_OW_NOISE_GRADIENT_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &PRELIM_SCALE_4_DENSITY,
    argument2: &PRELIM_OW_QUARTER_NEG_DENSITY,
};
// add(noiseGradient, constant(-0.703125)).clamp(-64.0, 64.0)
pub const PRELIM_INNER_DENSITY_OFFSET_DENSITY: DensityFunction =
    DensityFunction::Constant(-0.703125);
pub const PRELIM_OW_INNER_DENSITY_WITH_OFFSET: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_OW_NOISE_GRADIENT_DENSITY,
    argument2: &PRELIM_INNER_DENSITY_OFFSET_DENSITY,
};
pub const PRELIM_OW_INNER_DENSITY_CLAMPED: DensityFunction = DensityFunction::Clamp {
    input: &PRELIM_OW_INNER_DENSITY_WITH_OFFSET,
    min: -64.0,
    max: 64.0,
};
// slideOverworld(false, inner):
// slide(inner, -64, 384, 80, 64, -0.078125, 0, 24, 0.1171875)
// topFactor = yClampedGradient(240, 256, 1.0, 0.0) — shared with OVERWORLD_SLIDE_TOP_GRADIENT_DENSITY
// lerp(topFactor, -0.078125, inner) = add(mul(topFactor, add(inner, 0.078125)), -0.078125)
pub const PRELIM_OW_SLIDE_TOP_INNER_ADJUSTED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_OW_INNER_DENSITY_CLAMPED,
    argument2: &OVERWORLD_SLIDE_TOP_OFFSET_ADD_DENSITY,
};
pub const PRELIM_OW_SLIDE_TOP_MUL_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &OVERWORLD_SLIDE_TOP_GRADIENT_DENSITY,
    argument2: &PRELIM_OW_SLIDE_TOP_INNER_ADJUSTED_DENSITY,
};
pub const PRELIM_OW_TOP_SLIDE_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_OW_SLIDE_TOP_MUL_DENSITY,
    argument2: &OVERWORLD_SLIDE_TOP_NEG_TARGET_DENSITY,
};
// bottomFactor = yClampedGradient(-64, -40, 0.0, 1.0) — shared with OVERWORLD_SLIDE_BOTTOM_GRADIENT_DENSITY
// lerp(bottomFactor, 0.1171875, topSlide) = add(mul(bottomFactor, add(topSlide, -0.1171875)), 0.1171875)
pub const PRELIM_OW_SLIDE_BOTTOM_INNER_ADJUSTED_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Add,
        argument1: &PRELIM_OW_TOP_SLIDE_DENSITY,
        argument2: &OVERWORLD_SLIDE_BOTTOM_NEG_TARGET_DENSITY,
    };
pub const PRELIM_OW_SLIDE_BOTTOM_MUL_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &OVERWORLD_SLIDE_BOTTOM_GRADIENT_DENSITY,
    argument2: &PRELIM_OW_SLIDE_BOTTOM_INNER_ADJUSTED_DENSITY,
};
pub const PRELIM_OW_SLIDE_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_OW_SLIDE_BOTTOM_MUL_DENSITY,
    argument2: &OVERWORLD_SLIDE_BOTTOM_OFFSET_DENSITY,
};
// add(slide, constant(-0.390625))
pub const PRELIM_FINAL_SLIDE_OFFSET_DENSITY: DensityFunction = DensityFunction::Constant(-0.390625);
pub const PRELIM_OW_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_OW_SLIDE_DENSITY,
    argument2: &PRELIM_FINAL_SLIDE_OFFSET_DENSITY,
};
// findTopSurface(density, upperBound, -64, 8)
pub const OVERWORLD_PRELIMINARY_SURFACE_LEVEL_DENSITY: DensityFunction =
    DensityFunction::FindTopSurface {
        density: &PRELIM_OW_DENSITY,
        upper_bound: &PRELIM_OW_UPPER_BOUND_DENSITY,
        lower_bound: -64,
        cell_height: 8,
    };

// --- Large biomes preliminary_surface_level (amplified=false, largeBiomes=true) ---
// Identical structure to standard overworld but references large_biomes offset/factor.

pub const PRELIM_LB_FACTOR_REF_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld_large_biomes/factor");
pub const PRELIM_LB_OFFSET_REF_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld_large_biomes/offset");
pub const PRELIM_LB_CACHED_FACTOR_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Cache2D,
    input: &PRELIM_LB_FACTOR_REF_DENSITY,
};
pub const PRELIM_LB_CACHED_OFFSET_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Cache2D,
    input: &PRELIM_LB_OFFSET_REF_DENSITY,
};
pub const PRELIM_LB_FACTOR_INVERTED_DENSITY: DensityFunction = DensityFunction::Mapped {
    kind: MappedDensityFunction::Invert,
    input: &PRELIM_LB_CACHED_FACTOR_DENSITY,
};
pub const PRELIM_LB_SCALED_INVERT_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &PRELIM_OW_FACTOR_INVERT_SCALE_DENSITY,
    argument2: &PRELIM_LB_FACTOR_INVERTED_DENSITY,
};
pub const PRELIM_LB_NEG_CACHED_OFFSET_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &PRELIM_OW_NEG_OFFSET_SCALE_DENSITY,
    argument2: &PRELIM_LB_CACHED_OFFSET_DENSITY,
};
pub const PRELIM_LB_UPPER_BOUND_SUM_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_LB_SCALED_INVERT_DENSITY,
    argument2: &PRELIM_LB_NEG_CACHED_OFFSET_DENSITY,
};
pub const PRELIM_LB_UPPER_BOUND_SCALED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &PRELIM_LB_UPPER_BOUND_SUM_DENSITY,
    argument2: &PRELIM_REMAP_SCALE_DENSITY,
};
pub const PRELIM_LB_UPPER_BOUND_REMAPPED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_LB_UPPER_BOUND_SCALED_DENSITY,
    argument2: &PRELIM_REMAP_OFFSET_DENSITY,
};
pub const PRELIM_LB_UPPER_BOUND_DENSITY: DensityFunction = DensityFunction::Clamp {
    input: &PRELIM_LB_UPPER_BOUND_REMAPPED_DENSITY,
    min: -40.0,
    max: 320.0,
};
pub const PRELIM_LB_OFFSET_TO_DEPTH_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_DEPTH_GRADIENT_DENSITY,
    argument2: &PRELIM_LB_CACHED_OFFSET_DENSITY,
};
pub const PRELIM_LB_DEPTH_MUL_FACTOR_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &PRELIM_LB_OFFSET_TO_DEPTH_DENSITY,
    argument2: &PRELIM_LB_CACHED_FACTOR_DENSITY,
};
pub const PRELIM_LB_QUARTER_NEG_DENSITY: DensityFunction = DensityFunction::Mapped {
    kind: MappedDensityFunction::QuarterNegative,
    input: &PRELIM_LB_DEPTH_MUL_FACTOR_DENSITY,
};
pub const PRELIM_LB_NOISE_GRADIENT_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &PRELIM_SCALE_4_DENSITY,
    argument2: &PRELIM_LB_QUARTER_NEG_DENSITY,
};
pub const PRELIM_LB_INNER_DENSITY_WITH_OFFSET: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_LB_NOISE_GRADIENT_DENSITY,
    argument2: &PRELIM_INNER_DENSITY_OFFSET_DENSITY,
};
pub const PRELIM_LB_INNER_DENSITY_CLAMPED: DensityFunction = DensityFunction::Clamp {
    input: &PRELIM_LB_INNER_DENSITY_WITH_OFFSET,
    min: -64.0,
    max: 64.0,
};
// slide(inner, -64, 384, 80, 64, -0.078125, 0, 24, 0.1171875) — same as overworld
pub const PRELIM_LB_SLIDE_TOP_INNER_ADJUSTED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_LB_INNER_DENSITY_CLAMPED,
    argument2: &OVERWORLD_SLIDE_TOP_OFFSET_ADD_DENSITY,
};
pub const PRELIM_LB_SLIDE_TOP_MUL_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &OVERWORLD_SLIDE_TOP_GRADIENT_DENSITY,
    argument2: &PRELIM_LB_SLIDE_TOP_INNER_ADJUSTED_DENSITY,
};
pub const PRELIM_LB_TOP_SLIDE_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_LB_SLIDE_TOP_MUL_DENSITY,
    argument2: &OVERWORLD_SLIDE_TOP_NEG_TARGET_DENSITY,
};
pub const PRELIM_LB_SLIDE_BOTTOM_INNER_ADJUSTED_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Add,
        argument1: &PRELIM_LB_TOP_SLIDE_DENSITY,
        argument2: &OVERWORLD_SLIDE_BOTTOM_NEG_TARGET_DENSITY,
    };
pub const PRELIM_LB_SLIDE_BOTTOM_MUL_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &OVERWORLD_SLIDE_BOTTOM_GRADIENT_DENSITY,
    argument2: &PRELIM_LB_SLIDE_BOTTOM_INNER_ADJUSTED_DENSITY,
};
pub const PRELIM_LB_SLIDE_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_LB_SLIDE_BOTTOM_MUL_DENSITY,
    argument2: &OVERWORLD_SLIDE_BOTTOM_OFFSET_DENSITY,
};
pub const PRELIM_LB_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_LB_SLIDE_DENSITY,
    argument2: &PRELIM_FINAL_SLIDE_OFFSET_DENSITY,
};
pub const OVERWORLD_LARGE_BIOMES_PRELIMINARY_SURFACE_LEVEL_DENSITY: DensityFunction =
    DensityFunction::FindTopSurface {
        density: &PRELIM_LB_DENSITY,
        upper_bound: &PRELIM_LB_UPPER_BOUND_DENSITY,
        lower_bound: -64,
        cell_height: 8,
    };

// --- Amplified preliminary_surface_level (amplified=true) ---
// Uses overworld_amplified offset/factor; slide uses different topStartY=16, topEndY=0, bottomTarget=0.4.
// slideOverworld(true, inner) = slide(inner, -64, 384, 16, 0, -0.078125, 0, 24, 0.4)
//   topFactor    = yClampedGradient(-64+384-16, -64+384-0, 1.0, 0.0) = yClampedGradient(304, 320, 1.0, 0.0)
//   bottomFactor = yClampedGradient(-64, -40, 0.0, 1.0) — same as non-amplified
//   bottomTarget = 0.4 (not 0.1171875)

pub const PRELIM_AMP_FACTOR_REF_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld_amplified/factor");
pub const PRELIM_AMP_OFFSET_REF_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld_amplified/offset");
pub const PRELIM_AMP_CACHED_FACTOR_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Cache2D,
    input: &PRELIM_AMP_FACTOR_REF_DENSITY,
};
pub const PRELIM_AMP_CACHED_OFFSET_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Cache2D,
    input: &PRELIM_AMP_OFFSET_REF_DENSITY,
};
pub const PRELIM_AMP_FACTOR_INVERTED_DENSITY: DensityFunction = DensityFunction::Mapped {
    kind: MappedDensityFunction::Invert,
    input: &PRELIM_AMP_CACHED_FACTOR_DENSITY,
};
pub const PRELIM_AMP_SCALED_INVERT_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &PRELIM_OW_FACTOR_INVERT_SCALE_DENSITY,
    argument2: &PRELIM_AMP_FACTOR_INVERTED_DENSITY,
};
pub const PRELIM_AMP_NEG_CACHED_OFFSET_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &PRELIM_OW_NEG_OFFSET_SCALE_DENSITY,
    argument2: &PRELIM_AMP_CACHED_OFFSET_DENSITY,
};
pub const PRELIM_AMP_UPPER_BOUND_SUM_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_AMP_SCALED_INVERT_DENSITY,
    argument2: &PRELIM_AMP_NEG_CACHED_OFFSET_DENSITY,
};
pub const PRELIM_AMP_UPPER_BOUND_SCALED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &PRELIM_AMP_UPPER_BOUND_SUM_DENSITY,
    argument2: &PRELIM_REMAP_SCALE_DENSITY,
};
pub const PRELIM_AMP_UPPER_BOUND_REMAPPED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_AMP_UPPER_BOUND_SCALED_DENSITY,
    argument2: &PRELIM_REMAP_OFFSET_DENSITY,
};
pub const PRELIM_AMP_UPPER_BOUND_DENSITY: DensityFunction = DensityFunction::Clamp {
    input: &PRELIM_AMP_UPPER_BOUND_REMAPPED_DENSITY,
    min: -40.0,
    max: 320.0,
};
pub const PRELIM_AMP_OFFSET_TO_DEPTH_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_DEPTH_GRADIENT_DENSITY,
    argument2: &PRELIM_AMP_CACHED_OFFSET_DENSITY,
};
pub const PRELIM_AMP_DEPTH_MUL_FACTOR_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &PRELIM_AMP_OFFSET_TO_DEPTH_DENSITY,
    argument2: &PRELIM_AMP_CACHED_FACTOR_DENSITY,
};
pub const PRELIM_AMP_QUARTER_NEG_DENSITY: DensityFunction = DensityFunction::Mapped {
    kind: MappedDensityFunction::QuarterNegative,
    input: &PRELIM_AMP_DEPTH_MUL_FACTOR_DENSITY,
};
pub const PRELIM_AMP_NOISE_GRADIENT_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &PRELIM_SCALE_4_DENSITY,
    argument2: &PRELIM_AMP_QUARTER_NEG_DENSITY,
};
pub const PRELIM_AMP_INNER_DENSITY_WITH_OFFSET: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_AMP_NOISE_GRADIENT_DENSITY,
    argument2: &PRELIM_INNER_DENSITY_OFFSET_DENSITY,
};
pub const PRELIM_AMP_INNER_DENSITY_CLAMPED: DensityFunction = DensityFunction::Clamp {
    input: &PRELIM_AMP_INNER_DENSITY_WITH_OFFSET,
    min: -64.0,
    max: 64.0,
};
// Amplified slide: topFactor = yClampedGradient(304, 320, 1.0, 0.0)
pub const PRELIM_AMP_SLIDE_TOP_GRADIENT_DENSITY: DensityFunction =
    DensityFunction::YClampedGradient {
        from_y: 304,
        to_y: 320,
        from_value: 1.0,
        to_value: 0.0,
    };
// lerp(topFactor, -0.078125, inner) = add(mul(topFactor, add(inner, 0.078125)), -0.078125)
pub const PRELIM_AMP_SLIDE_TOP_INNER_ADJUSTED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_AMP_INNER_DENSITY_CLAMPED,
    argument2: &OVERWORLD_SLIDE_TOP_OFFSET_ADD_DENSITY, // 0.078125 shared
};
pub const PRELIM_AMP_SLIDE_TOP_MUL_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &PRELIM_AMP_SLIDE_TOP_GRADIENT_DENSITY,
    argument2: &PRELIM_AMP_SLIDE_TOP_INNER_ADJUSTED_DENSITY,
};
pub const PRELIM_AMP_TOP_SLIDE_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_AMP_SLIDE_TOP_MUL_DENSITY,
    argument2: &OVERWORLD_SLIDE_TOP_NEG_TARGET_DENSITY, // -0.078125 shared
};
// bottomFactor = yClampedGradient(-64, -40, 0.0, 1.0) — shared with OVERWORLD_SLIDE_BOTTOM_GRADIENT_DENSITY
// Amplified bottomTarget = 0.4
// lerp(bottomFactor, 0.4, topSlide) = add(mul(bottomFactor, add(topSlide, -0.4)), 0.4)
pub const PRELIM_AMP_SLIDE_BOTTOM_NEG_TARGET_DENSITY: DensityFunction =
    DensityFunction::Constant(-0.4);
pub const PRELIM_AMP_SLIDE_BOTTOM_OFFSET_DENSITY: DensityFunction = DensityFunction::Constant(0.4);
pub const PRELIM_AMP_SLIDE_BOTTOM_INNER_ADJUSTED_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Add,
        argument1: &PRELIM_AMP_TOP_SLIDE_DENSITY,
        argument2: &PRELIM_AMP_SLIDE_BOTTOM_NEG_TARGET_DENSITY,
    };
pub const PRELIM_AMP_SLIDE_BOTTOM_MUL_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &OVERWORLD_SLIDE_BOTTOM_GRADIENT_DENSITY, // shared
    argument2: &PRELIM_AMP_SLIDE_BOTTOM_INNER_ADJUSTED_DENSITY,
};
pub const PRELIM_AMP_SLIDE_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_AMP_SLIDE_BOTTOM_MUL_DENSITY,
    argument2: &PRELIM_AMP_SLIDE_BOTTOM_OFFSET_DENSITY,
};
pub const PRELIM_AMP_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &PRELIM_AMP_SLIDE_DENSITY,
    argument2: &PRELIM_FINAL_SLIDE_OFFSET_DENSITY,
};
pub const OVERWORLD_AMPLIFIED_PRELIMINARY_SURFACE_LEVEL_DENSITY: DensityFunction =
    DensityFunction::FindTopSurface {
        density: &PRELIM_AMP_DENSITY,
        upper_bound: &PRELIM_AMP_UPPER_BOUND_DENSITY,
        lower_bound: -64,
        cell_height: 8,
    };

