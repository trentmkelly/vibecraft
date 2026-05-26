use super::*;

pub const RIDGE_REFERENCE_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld/ridges");
pub const RIDGE_ABS_DENSITY: DensityFunction = DensityFunction::Mapped {
    kind: MappedDensityFunction::Abs,
    input: &RIDGE_REFERENCE_DENSITY,
};
pub const RIDGE_INNER_OFFSET_DENSITY: DensityFunction =
    DensityFunction::Constant(-0.6666666666666666);
pub const RIDGE_INNER_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &RIDGE_INNER_OFFSET_DENSITY,
    argument2: &RIDGE_ABS_DENSITY,
};
pub const RIDGE_INNER_ABS_DENSITY: DensityFunction = DensityFunction::Mapped {
    kind: MappedDensityFunction::Abs,
    input: &RIDGE_INNER_DENSITY,
};
pub const RIDGE_OUTER_OFFSET_DENSITY: DensityFunction =
    DensityFunction::Constant(-0.3333333333333333);
pub const RIDGE_OUTER_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &RIDGE_OUTER_OFFSET_DENSITY,
    argument2: &RIDGE_INNER_ABS_DENSITY,
};
pub const RIDGE_FOLDED_SCALE_DENSITY: DensityFunction = DensityFunction::Constant(-3.0);
pub const RIDGE_FOLDED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &RIDGE_FOLDED_SCALE_DENSITY,
    argument2: &RIDGE_OUTER_DENSITY,
};
pub const BLEND_ALPHA_CACHE_ONCE_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::CacheOnce,
    input: &BLEND_ALPHA_DENSITY,
};
pub const BLEND_ALPHA_DENSITY: DensityFunction = DensityFunction::BlendAlpha;
pub const BLEND_OFFSET_DENSITY: DensityFunction = DensityFunction::BlendOffset;
pub const ONE_DENSITY: DensityFunction = DensityFunction::Constant(1.0);
pub const BLEND_ALPHA_INVERT_SCALE_DENSITY: DensityFunction = DensityFunction::Constant(-1.0);
pub const BLEND_ALPHA_NEGATED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &BLEND_ALPHA_INVERT_SCALE_DENSITY,
    argument2: &BLEND_ALPHA_CACHE_ONCE_DENSITY,
};
pub const BLEND_ALPHA_INVERSE_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &ONE_DENSITY,
    argument2: &BLEND_ALPHA_NEGATED_DENSITY,
};
pub const OVERWORLD_OFFSET_BLEND_TARGET_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &BLEND_OFFSET_DENSITY,
    argument2: &BLEND_ALPHA_INVERSE_DENSITY,
};
pub const OVERWORLD_OFFSET_SPLINE_OFFSET_DENSITY: DensityFunction =
    DensityFunction::Constant(-0.5037500262260437);
pub const OVERWORLD_OFFSET_SPLINE_DENSITY: DensityFunction = DensityFunction::Spline {
    kind: TerrainSplineKind::Offset,
};
pub const OVERWORLD_OFFSET_SPLINE_WITH_OFFSET_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &OVERWORLD_OFFSET_SPLINE_OFFSET_DENSITY,
    argument2: &OVERWORLD_OFFSET_SPLINE_DENSITY,
};
pub const OVERWORLD_OFFSET_SPLINE_WEIGHTED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &OVERWORLD_OFFSET_SPLINE_WITH_OFFSET_DENSITY,
    argument2: &BLEND_ALPHA_CACHE_ONCE_DENSITY,
};
pub const OVERWORLD_OFFSET_BLENDED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &OVERWORLD_OFFSET_BLEND_TARGET_DENSITY,
    argument2: &OVERWORLD_OFFSET_SPLINE_WEIGHTED_DENSITY,
};
pub const OVERWORLD_OFFSET_CACHE_2D_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Cache2D,
    input: &OVERWORLD_OFFSET_BLENDED_DENSITY,
};
pub const OVERWORLD_OFFSET_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::FlatCache,
    input: &OVERWORLD_OFFSET_CACHE_2D_DENSITY,
};
pub const BLENDING_FACTOR_DENSITY: DensityFunction = DensityFunction::Constant(10.0);
pub const BLENDING_FACTOR_NEGATED_DENSITY: DensityFunction = DensityFunction::Constant(-10.0);
pub const OVERWORLD_FACTOR_SPLINE_DENSITY: DensityFunction = DensityFunction::Spline {
    kind: TerrainSplineKind::Factor,
};
pub const OVERWORLD_FACTOR_SPLINE_DELTA_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &BLENDING_FACTOR_NEGATED_DENSITY,
    argument2: &OVERWORLD_FACTOR_SPLINE_DENSITY,
};
pub const OVERWORLD_FACTOR_SPLINE_WEIGHTED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &BLEND_ALPHA_DENSITY,
    argument2: &OVERWORLD_FACTOR_SPLINE_DELTA_DENSITY,
};
pub const OVERWORLD_FACTOR_BLENDED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &BLENDING_FACTOR_DENSITY,
    argument2: &OVERWORLD_FACTOR_SPLINE_WEIGHTED_DENSITY,
};
pub const OVERWORLD_FACTOR_CACHE_2D_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Cache2D,
    input: &OVERWORLD_FACTOR_BLENDED_DENSITY,
};
pub const OVERWORLD_FACTOR_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::FlatCache,
    input: &OVERWORLD_FACTOR_CACHE_2D_DENSITY,
};
pub const BLENDING_JAGGEDNESS_DENSITY: DensityFunction = DensityFunction::Constant(0.0);
pub const BLENDING_JAGGEDNESS_NEGATED_DENSITY: DensityFunction = DensityFunction::Constant(-0.0);
pub const OVERWORLD_JAGGEDNESS_SPLINE_DENSITY: DensityFunction = DensityFunction::Spline {
    kind: TerrainSplineKind::Jaggedness,
};
pub const OVERWORLD_JAGGEDNESS_SPLINE_DELTA_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &BLENDING_JAGGEDNESS_NEGATED_DENSITY,
    argument2: &OVERWORLD_JAGGEDNESS_SPLINE_DENSITY,
};
pub const OVERWORLD_JAGGEDNESS_SPLINE_WEIGHTED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &BLEND_ALPHA_DENSITY,
    argument2: &OVERWORLD_JAGGEDNESS_SPLINE_DELTA_DENSITY,
};
pub const OVERWORLD_JAGGEDNESS_BLENDED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &BLENDING_JAGGEDNESS_DENSITY,
    argument2: &OVERWORLD_JAGGEDNESS_SPLINE_WEIGHTED_DENSITY,
};
pub const OVERWORLD_JAGGEDNESS_CACHE_2D_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Cache2D,
    input: &OVERWORLD_JAGGEDNESS_BLENDED_DENSITY,
};
pub const OVERWORLD_JAGGEDNESS_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::FlatCache,
    input: &OVERWORLD_JAGGEDNESS_CACHE_2D_DENSITY,
};
pub const OVERWORLD_LARGE_BIOMES_OFFSET_SPLINE_DENSITY: DensityFunction = DensityFunction::Spline {
    kind: TerrainSplineKind::LargeBiomesOffset,
};
pub const OVERWORLD_LARGE_BIOMES_OFFSET_SPLINE_WITH_OFFSET_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Add,
        argument1: &OVERWORLD_OFFSET_SPLINE_OFFSET_DENSITY,
        argument2: &OVERWORLD_LARGE_BIOMES_OFFSET_SPLINE_DENSITY,
    };
pub const OVERWORLD_LARGE_BIOMES_OFFSET_SPLINE_WEIGHTED_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Mul,
        argument1: &OVERWORLD_LARGE_BIOMES_OFFSET_SPLINE_WITH_OFFSET_DENSITY,
        argument2: &BLEND_ALPHA_CACHE_ONCE_DENSITY,
    };
pub const OVERWORLD_LARGE_BIOMES_OFFSET_BLENDED_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Add,
        argument1: &OVERWORLD_OFFSET_BLEND_TARGET_DENSITY,
        argument2: &OVERWORLD_LARGE_BIOMES_OFFSET_SPLINE_WEIGHTED_DENSITY,
    };
pub const OVERWORLD_LARGE_BIOMES_OFFSET_CACHE_2D_DENSITY: DensityFunction =
    DensityFunction::Marker {
        kind: DensityMarker::Cache2D,
        input: &OVERWORLD_LARGE_BIOMES_OFFSET_BLENDED_DENSITY,
    };
pub const OVERWORLD_LARGE_BIOMES_OFFSET_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::FlatCache,
    input: &OVERWORLD_LARGE_BIOMES_OFFSET_CACHE_2D_DENSITY,
};
pub const OVERWORLD_LARGE_BIOMES_FACTOR_SPLINE_DENSITY: DensityFunction = DensityFunction::Spline {
    kind: TerrainSplineKind::LargeBiomesFactor,
};
pub const OVERWORLD_LARGE_BIOMES_FACTOR_SPLINE_DELTA_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Add,
        argument1: &BLENDING_FACTOR_NEGATED_DENSITY,
        argument2: &OVERWORLD_LARGE_BIOMES_FACTOR_SPLINE_DENSITY,
    };
pub const OVERWORLD_LARGE_BIOMES_FACTOR_SPLINE_WEIGHTED_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Mul,
        argument1: &BLEND_ALPHA_DENSITY,
        argument2: &OVERWORLD_LARGE_BIOMES_FACTOR_SPLINE_DELTA_DENSITY,
    };
pub const OVERWORLD_LARGE_BIOMES_FACTOR_BLENDED_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Add,
        argument1: &BLENDING_FACTOR_DENSITY,
        argument2: &OVERWORLD_LARGE_BIOMES_FACTOR_SPLINE_WEIGHTED_DENSITY,
    };
pub const OVERWORLD_LARGE_BIOMES_FACTOR_CACHE_2D_DENSITY: DensityFunction =
    DensityFunction::Marker {
        kind: DensityMarker::Cache2D,
        input: &OVERWORLD_LARGE_BIOMES_FACTOR_BLENDED_DENSITY,
    };
pub const OVERWORLD_LARGE_BIOMES_FACTOR_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::FlatCache,
    input: &OVERWORLD_LARGE_BIOMES_FACTOR_CACHE_2D_DENSITY,
};
pub const OVERWORLD_LARGE_BIOMES_JAGGEDNESS_SPLINE_DENSITY: DensityFunction =
    DensityFunction::Spline {
        kind: TerrainSplineKind::LargeBiomesJaggedness,
    };
pub const OVERWORLD_LARGE_BIOMES_JAGGEDNESS_SPLINE_DELTA_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Add,
        argument1: &BLENDING_JAGGEDNESS_NEGATED_DENSITY,
        argument2: &OVERWORLD_LARGE_BIOMES_JAGGEDNESS_SPLINE_DENSITY,
    };
pub const OVERWORLD_LARGE_BIOMES_JAGGEDNESS_SPLINE_WEIGHTED_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Mul,
        argument1: &BLEND_ALPHA_DENSITY,
        argument2: &OVERWORLD_LARGE_BIOMES_JAGGEDNESS_SPLINE_DELTA_DENSITY,
    };
pub const OVERWORLD_LARGE_BIOMES_JAGGEDNESS_BLENDED_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Add,
        argument1: &BLENDING_JAGGEDNESS_DENSITY,
        argument2: &OVERWORLD_LARGE_BIOMES_JAGGEDNESS_SPLINE_WEIGHTED_DENSITY,
    };
pub const OVERWORLD_LARGE_BIOMES_JAGGEDNESS_CACHE_2D_DENSITY: DensityFunction =
    DensityFunction::Marker {
        kind: DensityMarker::Cache2D,
        input: &OVERWORLD_LARGE_BIOMES_JAGGEDNESS_BLENDED_DENSITY,
    };
pub const OVERWORLD_LARGE_BIOMES_JAGGEDNESS_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::FlatCache,
    input: &OVERWORLD_LARGE_BIOMES_JAGGEDNESS_CACHE_2D_DENSITY,
};
pub const OVERWORLD_AMPLIFIED_OFFSET_SPLINE_DENSITY: DensityFunction = DensityFunction::Spline {
    kind: TerrainSplineKind::AmplifiedOffset,
};
pub const OVERWORLD_AMPLIFIED_OFFSET_SPLINE_WITH_OFFSET_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Add,
        argument1: &OVERWORLD_OFFSET_SPLINE_OFFSET_DENSITY,
        argument2: &OVERWORLD_AMPLIFIED_OFFSET_SPLINE_DENSITY,
    };
pub const OVERWORLD_AMPLIFIED_OFFSET_SPLINE_WEIGHTED_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Mul,
        argument1: &OVERWORLD_AMPLIFIED_OFFSET_SPLINE_WITH_OFFSET_DENSITY,
        argument2: &BLEND_ALPHA_CACHE_ONCE_DENSITY,
    };
pub const OVERWORLD_AMPLIFIED_OFFSET_BLENDED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &OVERWORLD_OFFSET_BLEND_TARGET_DENSITY,
    argument2: &OVERWORLD_AMPLIFIED_OFFSET_SPLINE_WEIGHTED_DENSITY,
};
pub const OVERWORLD_AMPLIFIED_OFFSET_CACHE_2D_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Cache2D,
    input: &OVERWORLD_AMPLIFIED_OFFSET_BLENDED_DENSITY,
};
pub const OVERWORLD_AMPLIFIED_OFFSET_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::FlatCache,
    input: &OVERWORLD_AMPLIFIED_OFFSET_CACHE_2D_DENSITY,
};
pub const OVERWORLD_AMPLIFIED_FACTOR_SPLINE_DENSITY: DensityFunction = DensityFunction::Spline {
    kind: TerrainSplineKind::AmplifiedFactor,
};
pub const OVERWORLD_AMPLIFIED_FACTOR_SPLINE_DELTA_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Add,
        argument1: &BLENDING_FACTOR_NEGATED_DENSITY,
        argument2: &OVERWORLD_AMPLIFIED_FACTOR_SPLINE_DENSITY,
    };
pub const OVERWORLD_AMPLIFIED_FACTOR_SPLINE_WEIGHTED_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Mul,
        argument1: &BLEND_ALPHA_DENSITY,
        argument2: &OVERWORLD_AMPLIFIED_FACTOR_SPLINE_DELTA_DENSITY,
    };
pub const OVERWORLD_AMPLIFIED_FACTOR_BLENDED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &BLENDING_FACTOR_DENSITY,
    argument2: &OVERWORLD_AMPLIFIED_FACTOR_SPLINE_WEIGHTED_DENSITY,
};
pub const OVERWORLD_AMPLIFIED_FACTOR_CACHE_2D_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Cache2D,
    input: &OVERWORLD_AMPLIFIED_FACTOR_BLENDED_DENSITY,
};
pub const OVERWORLD_AMPLIFIED_FACTOR_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::FlatCache,
    input: &OVERWORLD_AMPLIFIED_FACTOR_CACHE_2D_DENSITY,
};
pub const OVERWORLD_AMPLIFIED_JAGGEDNESS_SPLINE_DENSITY: DensityFunction =
    DensityFunction::Spline {
        kind: TerrainSplineKind::AmplifiedJaggedness,
    };
pub const OVERWORLD_AMPLIFIED_JAGGEDNESS_SPLINE_DELTA_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Add,
        argument1: &BLENDING_JAGGEDNESS_NEGATED_DENSITY,
        argument2: &OVERWORLD_AMPLIFIED_JAGGEDNESS_SPLINE_DENSITY,
    };
pub const OVERWORLD_AMPLIFIED_JAGGEDNESS_SPLINE_WEIGHTED_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Mul,
        argument1: &BLEND_ALPHA_DENSITY,
        argument2: &OVERWORLD_AMPLIFIED_JAGGEDNESS_SPLINE_DELTA_DENSITY,
    };
pub const OVERWORLD_AMPLIFIED_JAGGEDNESS_BLENDED_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Add,
        argument1: &BLENDING_JAGGEDNESS_DENSITY,
        argument2: &OVERWORLD_AMPLIFIED_JAGGEDNESS_SPLINE_WEIGHTED_DENSITY,
    };
pub const OVERWORLD_AMPLIFIED_JAGGEDNESS_CACHE_2D_DENSITY: DensityFunction =
    DensityFunction::Marker {
        kind: DensityMarker::Cache2D,
        input: &OVERWORLD_AMPLIFIED_JAGGEDNESS_BLENDED_DENSITY,
    };
pub const OVERWORLD_AMPLIFIED_JAGGEDNESS_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::FlatCache,
    input: &OVERWORLD_AMPLIFIED_JAGGEDNESS_CACHE_2D_DENSITY,
};
pub const OVERWORLD_DEPTH_GRADIENT_DENSITY: DensityFunction = DensityFunction::YClampedGradient {
    from_y: -64,
    to_y: 320,
    from_value: 1.5,
    to_value: -1.5,
};
pub const OVERWORLD_OFFSET_REFERENCE_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld/offset");
pub const OVERWORLD_DEPTH_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &OVERWORLD_DEPTH_GRADIENT_DENSITY,
    argument2: &OVERWORLD_OFFSET_REFERENCE_DENSITY,
};
pub const OVERWORLD_LARGE_BIOMES_OFFSET_REFERENCE_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld_large_biomes/offset");
pub const OVERWORLD_LARGE_BIOMES_DEPTH_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &OVERWORLD_DEPTH_GRADIENT_DENSITY,
    argument2: &OVERWORLD_LARGE_BIOMES_OFFSET_REFERENCE_DENSITY,
};
pub const OVERWORLD_AMPLIFIED_OFFSET_REFERENCE_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld_amplified/offset");
pub const OVERWORLD_AMPLIFIED_DEPTH_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &OVERWORLD_DEPTH_GRADIENT_DENSITY,
    argument2: &OVERWORLD_AMPLIFIED_OFFSET_REFERENCE_DENSITY,
};
pub const JAGGED_NOISE_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:jagged",
    xz_scale: 1500.0,
    y_scale: 0.0,
};
pub const JAGGED_HALF_NEGATIVE_DENSITY: DensityFunction = DensityFunction::Mapped {
    kind: MappedDensityFunction::HalfNegative,
    input: &JAGGED_NOISE_DENSITY,
};
pub const OVERWORLD_DEPTH_REFERENCE_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld/depth");
pub const OVERWORLD_JAGGEDNESS_REFERENCE_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld/jaggedness");
pub const OVERWORLD_JAGGEDNESS_NOISE_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &OVERWORLD_JAGGEDNESS_REFERENCE_DENSITY,
    argument2: &JAGGED_HALF_NEGATIVE_DENSITY,
};
pub const OVERWORLD_DEPTH_WITH_JAGGEDNESS_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &OVERWORLD_DEPTH_REFERENCE_DENSITY,
    argument2: &OVERWORLD_JAGGEDNESS_NOISE_DENSITY,
};
pub const OVERWORLD_FACTOR_REFERENCE_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld/factor");
pub const OVERWORLD_SLOPED_CHEESE_INPUT_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &OVERWORLD_DEPTH_WITH_JAGGEDNESS_DENSITY,
    argument2: &OVERWORLD_FACTOR_REFERENCE_DENSITY,
};
pub const OVERWORLD_SLOPED_CHEESE_QUARTER_DENSITY: DensityFunction = DensityFunction::Mapped {
    kind: MappedDensityFunction::QuarterNegative,
    input: &OVERWORLD_SLOPED_CHEESE_INPUT_DENSITY,
};
pub const SLOPED_CHEESE_SCALE_DENSITY: DensityFunction = DensityFunction::Constant(4.0);
pub const OVERWORLD_SLOPED_CHEESE_SCALED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &SLOPED_CHEESE_SCALE_DENSITY,
    argument2: &OVERWORLD_SLOPED_CHEESE_QUARTER_DENSITY,
};
pub const OVERWORLD_BASE_3D_NOISE_REFERENCE_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld/base_3d_noise");
pub const OVERWORLD_SLOPED_CHEESE_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &OVERWORLD_SLOPED_CHEESE_SCALED_DENSITY,
    argument2: &OVERWORLD_BASE_3D_NOISE_REFERENCE_DENSITY,
};
pub const OVERWORLD_LARGE_BIOMES_DEPTH_REFERENCE_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld_large_biomes/depth");
pub const OVERWORLD_LARGE_BIOMES_JAGGEDNESS_REFERENCE_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld_large_biomes/jaggedness");
pub const OVERWORLD_LARGE_BIOMES_JAGGEDNESS_NOISE_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Mul,
        argument1: &OVERWORLD_LARGE_BIOMES_JAGGEDNESS_REFERENCE_DENSITY,
        argument2: &JAGGED_HALF_NEGATIVE_DENSITY,
    };
pub const OVERWORLD_LARGE_BIOMES_DEPTH_WITH_JAGGEDNESS_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Add,
        argument1: &OVERWORLD_LARGE_BIOMES_DEPTH_REFERENCE_DENSITY,
        argument2: &OVERWORLD_LARGE_BIOMES_JAGGEDNESS_NOISE_DENSITY,
    };
pub const OVERWORLD_LARGE_BIOMES_FACTOR_REFERENCE_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld_large_biomes/factor");
pub const OVERWORLD_LARGE_BIOMES_SLOPED_CHEESE_INPUT_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Mul,
        argument1: &OVERWORLD_LARGE_BIOMES_DEPTH_WITH_JAGGEDNESS_DENSITY,
        argument2: &OVERWORLD_LARGE_BIOMES_FACTOR_REFERENCE_DENSITY,
    };
pub const OVERWORLD_LARGE_BIOMES_SLOPED_CHEESE_QUARTER_DENSITY: DensityFunction =
    DensityFunction::Mapped {
        kind: MappedDensityFunction::QuarterNegative,
        input: &OVERWORLD_LARGE_BIOMES_SLOPED_CHEESE_INPUT_DENSITY,
    };
pub const OVERWORLD_LARGE_BIOMES_SLOPED_CHEESE_SCALED_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Mul,
        argument1: &SLOPED_CHEESE_SCALE_DENSITY,
        argument2: &OVERWORLD_LARGE_BIOMES_SLOPED_CHEESE_QUARTER_DENSITY,
    };
pub const OVERWORLD_LARGE_BIOMES_SLOPED_CHEESE_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &OVERWORLD_LARGE_BIOMES_SLOPED_CHEESE_SCALED_DENSITY,
    argument2: &OVERWORLD_BASE_3D_NOISE_REFERENCE_DENSITY,
};
pub const OVERWORLD_AMPLIFIED_DEPTH_REFERENCE_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld_amplified/depth");
pub const OVERWORLD_AMPLIFIED_JAGGEDNESS_REFERENCE_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld_amplified/jaggedness");
pub const OVERWORLD_AMPLIFIED_JAGGEDNESS_NOISE_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &OVERWORLD_AMPLIFIED_JAGGEDNESS_REFERENCE_DENSITY,
    argument2: &JAGGED_HALF_NEGATIVE_DENSITY,
};
pub const OVERWORLD_AMPLIFIED_DEPTH_WITH_JAGGEDNESS_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Add,
        argument1: &OVERWORLD_AMPLIFIED_DEPTH_REFERENCE_DENSITY,
        argument2: &OVERWORLD_AMPLIFIED_JAGGEDNESS_NOISE_DENSITY,
    };
pub const OVERWORLD_AMPLIFIED_FACTOR_REFERENCE_DENSITY: DensityFunction =
    DensityFunction::Reference("minecraft:overworld_amplified/factor");
pub const OVERWORLD_AMPLIFIED_SLOPED_CHEESE_INPUT_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Mul,
        argument1: &OVERWORLD_AMPLIFIED_DEPTH_WITH_JAGGEDNESS_DENSITY,
        argument2: &OVERWORLD_AMPLIFIED_FACTOR_REFERENCE_DENSITY,
    };
pub const OVERWORLD_AMPLIFIED_SLOPED_CHEESE_QUARTER_DENSITY: DensityFunction =
    DensityFunction::Mapped {
        kind: MappedDensityFunction::QuarterNegative,
        input: &OVERWORLD_AMPLIFIED_SLOPED_CHEESE_INPUT_DENSITY,
    };
pub const OVERWORLD_AMPLIFIED_SLOPED_CHEESE_SCALED_DENSITY: DensityFunction =
    DensityFunction::Binary {
        kind: BinaryDensityFunction::Mul,
        argument1: &SLOPED_CHEESE_SCALE_DENSITY,
        argument2: &OVERWORLD_AMPLIFIED_SLOPED_CHEESE_QUARTER_DENSITY,
    };
pub const OVERWORLD_AMPLIFIED_SLOPED_CHEESE_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &OVERWORLD_AMPLIFIED_SLOPED_CHEESE_SCALED_DENSITY,
    argument2: &OVERWORLD_BASE_3D_NOISE_REFERENCE_DENSITY,
};
