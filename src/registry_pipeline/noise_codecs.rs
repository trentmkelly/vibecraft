//! Codecs for `worldgen/noise` and `worldgen/density_function`.
//!
//! - `NormalNoise.NoiseParameters.DIRECT_CODEC`
//! - `DensityFunctions.DIRECT_CODEC` and every registered density function type
//!   (`DensityFunctions.bootstrap`), including the marker/wrapping types, the
//!   two-argument functions and the `CubicSpline` used by `spline`.

use crate::registry_pipeline::codec::{
    self, double_codec, either, enum_codec, float_codec, holder_file, int_range, lazy, list,
    positive_int, record, req, Codec,
};
use crate::registry_pipeline::worldgen_common::{
    collapse_variant, double_in, non_empty_list, registry_dispatch, unit, Variant, MAX_Y, MIN_Y,
};

/// `Registries.NOISE`.
pub const NOISE_REGISTRY: &str = "minecraft:worldgen/noise";
/// `Registries.DENSITY_FUNCTION`.
pub const DENSITY_FUNCTION_REGISTRY: &str = "minecraft:worldgen/density_function";

/// `DensityFunctions.MAX_REASONABLE_NOISE_VALUE`.
const MAX_REASONABLE_NOISE_VALUE: f64 = 1_000_000.0;

/// `NormalNoise.NoiseParameters.DIRECT_CODEC`.
pub fn noise_parameters() -> Codec {
    record(vec![
        req("firstOctave", codec::int_codec()),
        req("amplitudes", list(double_codec())),
    ])
}

/// `NormalNoise.NoiseParameters.CODEC` (`RegistryFileCodec`): a reference to a
/// `worldgen/noise` entry or an inline definition.
pub fn noise_holder() -> Codec {
    holder_file(NOISE_REGISTRY, noise_parameters())
}

/// `DensityFunctions.NOISE_VALUE_CODEC`.
fn noise_value() -> Codec {
    double_in(-MAX_REASONABLE_NOISE_VALUE, MAX_REASONABLE_NOISE_VALUE)
}

/// `DensityFunction.HOLDER_HELPER_CODEC`: a `worldgen/density_function` reference or an
/// inline function.
pub fn density_function_holder() -> Codec {
    holder_file(DENSITY_FUNCTION_REGISTRY, lazy(density_function))
}

/// `DensityFunction.DIRECT_CODEC` / `DensityFunctions.DIRECT_CODEC`: a bare constant
/// or a typed function. A `constant` function is written back as its number.
pub fn density_function() -> Codec {
    let typed = registry_dispatch(
        "type",
        "minecraft:worldgen/density_function_type",
        DENSITY_FUNCTIONS,
    );
    either(noise_value(), typed).map_tag(|tag| Ok(collapse_variant(tag, "constant", "argument")))
}

/// `DensityFunctions.bootstrap` registrations, in registration order.
const DENSITY_FUNCTIONS: &[Variant] = &[
    ("blend_alpha", unit),
    ("blend_offset", unit),
    ("beardifier", unit),
    ("old_blended_noise", blended_noise),
    ("interpolated", single_function_argument),
    ("flat_cache", single_function_argument),
    ("cache_2d", single_function_argument),
    ("cache_once", single_function_argument),
    ("cache_all_in_cell", single_function_argument),
    ("noise", noise),
    ("end_islands", unit),
    ("weird_scaled_sampler", weird_scaled_sampler),
    ("shifted_noise", shifted_noise),
    ("range_choice", range_choice),
    ("shift_a", noise_argument),
    ("shift_b", noise_argument),
    ("shift", noise_argument),
    ("blend_density", single_function_argument),
    ("clamp", clamp),
    ("abs", single_function_argument),
    ("square", single_function_argument),
    ("cube", single_function_argument),
    ("half_negative", single_function_argument),
    ("quarter_negative", single_function_argument),
    ("invert", single_function_argument),
    ("squeeze", single_function_argument),
    ("add", two_arguments),
    ("mul", two_arguments),
    ("min", two_arguments),
    ("max", two_arguments),
    ("spline", spline),
    ("constant", constant),
    ("y_clamped_gradient", y_clamped_gradient),
    ("find_top_surface", find_top_surface),
];

/// `BlendedNoise.DATA_CODEC`.
fn blended_noise() -> Codec {
    let scale = || double_in(0.001, 1000.0);
    record(vec![
        req("xz_scale", scale()),
        req("y_scale", scale()),
        req("xz_factor", scale()),
        req("y_factor", scale()),
        req("smear_scale_multiplier", double_in(1.0, 8.0)),
    ])
}

/// `DensityFunctions.singleFunctionArgumentCodec`.
fn single_function_argument() -> Codec {
    record(vec![req("argument", density_function_holder())])
}

/// `DensityFunctions.singleArgumentCodec(NoiseHolder.CODEC, ...)` (`shift*`).
fn noise_argument() -> Codec {
    record(vec![req("argument", noise_holder())])
}

/// `DensityFunctions.doubleFunctionArgumentCodec` (`add`/`mul`/`min`/`max`).
fn two_arguments() -> Codec {
    record(vec![
        req("argument1", density_function_holder()),
        req("argument2", density_function_holder()),
    ])
}

/// `DensityFunctions.Noise.DATA_CODEC`.
fn noise() -> Codec {
    record(vec![
        req("noise", noise_holder()),
        req("xz_scale", double_codec()),
        req("y_scale", double_codec()),
    ])
}

/// `DensityFunctions.WeirdScaledSampler.DATA_CODEC`.
fn weird_scaled_sampler() -> Codec {
    record(vec![
        req("input", density_function_holder()),
        req("noise", noise_holder()),
        req("rarity_value_mapper", enum_codec(&["type_1", "type_2"])),
    ])
}

/// `DensityFunctions.ShiftedNoise.DATA_CODEC`.
fn shifted_noise() -> Codec {
    record(vec![
        req("shift_x", density_function_holder()),
        req("shift_y", density_function_holder()),
        req("shift_z", density_function_holder()),
        req("xz_scale", double_codec()),
        req("y_scale", double_codec()),
        req("noise", noise_holder()),
    ])
}

/// `DensityFunctions.RangeChoice.DATA_CODEC`.
fn range_choice() -> Codec {
    record(vec![
        req("input", density_function_holder()),
        req("min_inclusive", noise_value()),
        req("max_exclusive", noise_value()),
        req("when_in_range", density_function_holder()),
        req("when_out_of_range", density_function_holder()),
    ])
}

/// `DensityFunctions.Clamp.DATA_CODEC`: the input is a direct (never referenced)
/// function.
fn clamp() -> Codec {
    record(vec![
        req("input", lazy(density_function)),
        req("min", noise_value()),
        req("max", noise_value()),
    ])
}

/// `DensityFunctions.Constant.CODEC`.
fn constant() -> Codec {
    record(vec![req("argument", noise_value())])
}

/// `DensityFunctions.YClampedGradient.DATA_CODEC`.
fn y_clamped_gradient() -> Codec {
    let y = || int_range(MIN_Y * 2, MAX_Y * 2);
    record(vec![
        req("from_y", y()),
        req("to_y", y()),
        req("from_value", noise_value()),
        req("to_value", noise_value()),
    ])
}

/// `DensityFunctions.FindTopSurface.DATA_CODEC`.
fn find_top_surface() -> Codec {
    record(vec![
        req("density", density_function_holder()),
        req("upper_bound", density_function_holder()),
        req("lower_bound", int_range(MIN_Y * 2, MAX_Y * 2)),
        req("cell_height", positive_int()),
    ])
}

/// `DensityFunctions.Spline.DATA_CODEC`: `{spline: <CubicSpline>}`.
fn spline() -> Codec {
    record(vec![req("spline", cubic_spline())])
}

/// `CubicSpline.codec(Coordinate.CODEC)`: a bare float (a constant spline) or a
/// multipoint spline with a density-function coordinate.
fn cubic_spline() -> Codec {
    let multipoint = record(vec![
        req("coordinate", density_function_holder()),
        req("points", non_empty_list(spline_point())),
    ]);
    either(float_codec(), multipoint)
}

/// One `CubicSpline` point: `{location, value: <spline>, derivative}`.
fn spline_point() -> Codec {
    record(vec![
        req("location", float_codec()),
        req("value", lazy(cubic_spline)),
        req("derivative", float_codec()),
    ])
}
