use super::*;

#[derive(Debug, Clone, Copy)]
pub(super) struct TerrainSplineContext {
    pub(super) continents: f64,
    pub(super) erosion: f64,
    pub(super) weirdness: f64,
    pub(super) ridges: f64,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum TerrainSplineCoordinate {
    Continents,
    Erosion,
    Weirdness,
    Ridges,
}

impl TerrainSplineCoordinate {
    fn apply(self, context: TerrainSplineContext) -> f64 {
        match self {
            Self::Continents => context.continents,
            Self::Erosion => context.erosion,
            Self::Weirdness => context.weirdness,
            Self::Ridges => context.ridges,
        }
    }

    fn bounds(self) -> (f64, f64) {
        (-2.0, 2.0)
    }
}

#[derive(Debug, Clone)]
pub(super) enum TerrainCubicSpline {
    Constant(f64),
    Multipoint {
        coordinate: TerrainSplineCoordinate,
        locations: Vec<f64>,
        values: Vec<TerrainCubicSpline>,
        derivatives: Vec<f64>,
    },
}

impl TerrainCubicSpline {
    pub(super) fn apply(&self, context: TerrainSplineContext) -> f64 {
        match self {
            Self::Constant(value) => *value,
            Self::Multipoint {
                coordinate,
                locations,
                values,
                derivatives,
            } => {
                let input = coordinate.apply(context);
                let start = locations.partition_point(|location| input >= *location);
                if start == 0 {
                    return linear_extend(
                        input,
                        locations[0],
                        values[0].apply(context),
                        derivatives[0],
                    );
                }
                let index = start - 1;
                let last_index = locations.len() - 1;
                if index == last_index {
                    return linear_extend(
                        input,
                        locations[last_index],
                        values[last_index].apply(context),
                        derivatives[last_index],
                    );
                }

                let x1 = locations[index];
                let x2 = locations[index + 1];
                let t = (input - x1) / (x2 - x1);
                let y1 = values[index].apply(context);
                let y2 = values[index + 1].apply(context);
                let d1 = derivatives[index];
                let d2 = derivatives[index + 1];
                let a = d1 * (x2 - x1) - (y2 - y1);
                let b = -d2 * (x2 - x1) + (y2 - y1);
                lerp(t, y1, y2) + t * (1.0 - t) * lerp(t, a, b)
            }
        }
    }

    pub(super) fn bounds(&self) -> (f64, f64) {
        match self {
            Self::Constant(value) => (*value, *value),
            Self::Multipoint {
                coordinate,
                locations,
                values,
                derivatives,
            } => {
                let last_index = locations.len() - 1;
                let (min_input, max_input) = coordinate.bounds();
                let mut min_value = f64::INFINITY;
                let mut max_value = f64::NEG_INFINITY;
                if min_input < locations[0] {
                    let (min, max) = values[0].bounds();
                    for edge in [
                        linear_extend(min_input, locations[0], min, derivatives[0]),
                        linear_extend(min_input, locations[0], max, derivatives[0]),
                    ] {
                        min_value = min_value.min(edge);
                        max_value = max_value.max(edge);
                    }
                }
                if max_input > locations[last_index] {
                    let (min, max) = values[last_index].bounds();
                    for edge in [
                        linear_extend(
                            max_input,
                            locations[last_index],
                            min,
                            derivatives[last_index],
                        ),
                        linear_extend(
                            max_input,
                            locations[last_index],
                            max,
                            derivatives[last_index],
                        ),
                    ] {
                        min_value = min_value.min(edge);
                        max_value = max_value.max(edge);
                    }
                }
                for value in values {
                    let (min, max) = value.bounds();
                    min_value = min_value.min(min);
                    max_value = max_value.max(max);
                }
                for i in 0..last_index {
                    let x_diff = locations[i + 1] - locations[i];
                    let (min1, max1) = values[i].bounds();
                    let (min2, max2) = values[i + 1].bounds();
                    let d1 = derivatives[i];
                    let d2 = derivatives[i + 1];
                    if d1 != 0.0 || d2 != 0.0 {
                        let p1 = d1 * x_diff;
                        let p2 = d2 * x_diff;
                        let min_lerp1 = min1.min(min2);
                        let max_lerp1 = max1.max(max2);
                        let min_a = p1 - max2 + min1;
                        let max_a = p1 - min2 + max1;
                        let min_b = -p2 + min2 - max1;
                        let max_b = -p2 + max2 - min1;
                        min_value = min_value.min(min_lerp1 + 0.25 * min_a.min(min_b));
                        max_value = max_value.max(max_lerp1 + 0.25 * max_a.max(max_b));
                    }
                }
                (min_value, max_value)
            }
        }
    }
}

#[derive(Debug, Clone)]
struct TerrainSplineBuilder {
    coordinate: TerrainSplineCoordinate,
    transformer: fn(f64) -> f64,
    locations: Vec<f64>,
    values: Vec<TerrainCubicSpline>,
    derivatives: Vec<f64>,
}

impl TerrainSplineBuilder {
    fn new(coordinate: TerrainSplineCoordinate, transformer: fn(f64) -> f64) -> Self {
        Self {
            coordinate,
            transformer,
            locations: Vec::new(),
            values: Vec::new(),
            derivatives: Vec::new(),
        }
    }

    fn point(mut self, location: f64, value: f64) -> Self {
        self.push(
            location,
            TerrainCubicSpline::Constant((self.transformer)(value)),
            0.0,
        );
        self
    }

    fn point_derivative(mut self, location: f64, value: f64, derivative: f64) -> Self {
        self.push(
            location,
            TerrainCubicSpline::Constant((self.transformer)(value)),
            derivative,
        );
        self
    }

    fn spline(mut self, location: f64, value: TerrainCubicSpline) -> Self {
        self.push(location, value, 0.0);
        self
    }

    fn push(&mut self, location: f64, value: TerrainCubicSpline, derivative: f64) {
        if let Some(previous) = self.locations.last() {
            assert!(location > *previous, "terrain spline points must ascend");
        }
        self.locations.push(location);
        self.values.push(value);
        self.derivatives.push(derivative);
    }

    fn build(self) -> TerrainCubicSpline {
        TerrainCubicSpline::Multipoint {
            coordinate: self.coordinate,
            locations: self.locations,
            values: self.values,
            derivatives: self.derivatives,
        }
    }
}

fn linear_extend(input: f64, location: f64, value: f64, derivative: f64) -> f64 {
    if derivative == 0.0 {
        value
    } else {
        value + derivative * (input - location)
    }
}

fn no_transform(value: f64) -> f64 {
    value
}

fn amplified_offset(value: f64) -> f64 {
    if value < 0.0 {
        value
    } else {
        value * 2.0
    }
}

fn amplified_factor(value: f64) -> f64 {
    1.25 - 6.25 / (value + 5.0)
}

fn amplified_jaggedness(value: f64) -> f64 {
    value * 2.0
}

pub(super) fn peaks_and_valleys(weirdness: f64) -> f64 {
    -((weirdness.abs() - 0.6666667).abs() - 0.33333334) * 3.0
}

pub(super) fn terrain_spline_context(
    seed: i64,
    settings: NoiseGeneratorSettings,
    block_x: i32,
    block_y: i32,
    block_z: i32,
) -> TerrainSplineContext {
    let continents = DensityFunction::Reference("minecraft:overworld/continents")
        .compute_with_noise(seed, settings, block_x, block_y, block_z);
    let erosion = DensityFunction::Reference("minecraft:overworld/erosion")
        .compute_with_noise(seed, settings, block_x, block_y, block_z);
    let weirdness = DensityFunction::Reference("minecraft:overworld/ridges")
        .compute_with_noise(seed, settings, block_x, block_y, block_z);
    TerrainSplineContext {
        continents,
        erosion,
        weirdness,
        ridges: peaks_and_valleys(weirdness),
    }
}

pub(super) fn terrain_spline(kind: TerrainSplineKind) -> TerrainCubicSpline {
    match kind {
        TerrainSplineKind::OverworldOffset | TerrainSplineKind::OverworldLargeBiomesOffset => {
            overworld_offset_spline(false)
        }
        TerrainSplineKind::OverworldAmplifiedOffset => overworld_offset_spline(true),
        TerrainSplineKind::OverworldFactor | TerrainSplineKind::OverworldLargeBiomesFactor => {
            overworld_factor_spline(false)
        }
        TerrainSplineKind::OverworldAmplifiedFactor => overworld_factor_spline(true),
        TerrainSplineKind::OverworldJaggedness
        | TerrainSplineKind::OverworldLargeBiomesJaggedness => overworld_jaggedness_spline(false),
        TerrainSplineKind::OverworldAmplifiedJaggedness => overworld_jaggedness_spline(true),
    }
}

fn overworld_offset_spline(amplified: bool) -> TerrainCubicSpline {
    let transformer = if amplified {
        amplified_offset
    } else {
        no_transform
    };
    let beach = erosion_offset_spline(-0.15, 0.0, 0.0, 0.1, 0.0, -0.03, false, false, transformer);
    let low = erosion_offset_spline(-0.1, 0.03, 0.1, 0.1, 0.01, -0.03, false, false, transformer);
    let mid = erosion_offset_spline(-0.1, 0.03, 0.1, 0.7, 0.01, -0.03, true, true, transformer);
    let high = erosion_offset_spline(-0.05, 0.03, 0.1, 1.0, 0.01, 0.01, true, true, transformer);
    TerrainSplineBuilder::new(TerrainSplineCoordinate::Continents, transformer)
        .point(-1.1, 0.044)
        .point(-1.02, -0.2222)
        .point(-0.51, -0.2222)
        .point(-0.44, -0.12)
        .point(-0.18, -0.12)
        .spline(-0.16, beach.clone())
        .spline(-0.15, beach)
        .spline(-0.1, low)
        .spline(0.25, mid)
        .spline(1.0, high)
        .build()
}

fn overworld_factor_spline(amplified: bool) -> TerrainCubicSpline {
    let transformer = if amplified {
        amplified_factor
    } else {
        no_transform
    };
    TerrainSplineBuilder::new(TerrainSplineCoordinate::Continents, no_transform)
        .point(-0.19, 3.95)
        .spline(-0.15, erosion_factor_spline(6.25, true, no_transform))
        .spline(-0.1, erosion_factor_spline(5.47, true, transformer))
        .spline(0.03, erosion_factor_spline(5.08, true, transformer))
        .spline(0.06, erosion_factor_spline(4.69, false, transformer))
        .build()
}

fn overworld_jaggedness_spline(amplified: bool) -> TerrainCubicSpline {
    let transformer = if amplified {
        amplified_jaggedness
    } else {
        no_transform
    };
    TerrainSplineBuilder::new(TerrainSplineCoordinate::Continents, transformer)
        .point(-0.11, 0.0)
        .spline(
            0.03,
            erosion_jaggedness_spline(1.0, 0.5, 0.0, 0.0, transformer),
        )
        .spline(
            0.65,
            erosion_jaggedness_spline(1.0, 1.0, 1.0, 0.0, transformer),
        )
        .build()
}

fn erosion_jaggedness_spline(
    peak0: f64,
    peak1: f64,
    high0: f64,
    high1: f64,
    transformer: fn(f64) -> f64,
) -> TerrainCubicSpline {
    let ridge0 = ridge_jaggedness_spline(peak0, high0, transformer);
    let ridge1 = ridge_jaggedness_spline(peak1, high1, transformer);
    TerrainSplineBuilder::new(TerrainSplineCoordinate::Erosion, transformer)
        .spline(-1.0, ridge0)
        .spline(-0.78, ridge1.clone())
        .spline(-0.5775, ridge1)
        .point(-0.375, 0.0)
        .build()
}

fn ridge_jaggedness_spline(
    peak_factor: f64,
    high_factor: f64,
    transformer: fn(f64) -> f64,
) -> TerrainCubicSpline {
    let high_start = peaks_and_valleys(0.4);
    let high_end = peaks_and_valleys(0.56666666);
    let high_middle = (high_start + high_end) / 2.0;
    let mut builder = TerrainSplineBuilder::new(TerrainSplineCoordinate::Ridges, transformer)
        .point(high_start, 0.0);
    builder = if high_factor > 0.0 {
        builder.spline(
            high_middle,
            weirdness_jaggedness_spline(high_factor, transformer),
        )
    } else {
        builder.point(high_middle, 0.0)
    };
    if peak_factor > 0.0 {
        builder.spline(1.0, weirdness_jaggedness_spline(peak_factor, transformer))
    } else {
        builder.point(1.0, 0.0)
    }
    .build()
}

fn weirdness_jaggedness_spline(factor: f64, transformer: fn(f64) -> f64) -> TerrainCubicSpline {
    TerrainSplineBuilder::new(TerrainSplineCoordinate::Weirdness, transformer)
        .point(-0.01, 0.63 * factor)
        .point(0.01, 0.3 * factor)
        .build()
}

fn erosion_factor_spline(
    base_value: f64,
    shattered: bool,
    transformer: fn(f64) -> f64,
) -> TerrainCubicSpline {
    let base = TerrainSplineBuilder::new(TerrainSplineCoordinate::Weirdness, transformer)
        .point(-0.2, 6.3)
        .point(0.2, base_value)
        .build();
    let mut builder = TerrainSplineBuilder::new(TerrainSplineCoordinate::Erosion, transformer)
        .spline(-0.6, base.clone())
        .spline(
            -0.5,
            TerrainSplineBuilder::new(TerrainSplineCoordinate::Weirdness, transformer)
                .point(-0.05, 6.3)
                .point(0.05, 2.67)
                .build(),
        )
        .spline(-0.35, base.clone())
        .spline(-0.25, base.clone())
        .spline(
            -0.1,
            TerrainSplineBuilder::new(TerrainSplineCoordinate::Weirdness, transformer)
                .point(-0.05, 2.67)
                .point(0.05, 6.3)
                .build(),
        )
        .spline(0.03, base.clone());
    if shattered {
        let weirdness_shattered =
            TerrainSplineBuilder::new(TerrainSplineCoordinate::Weirdness, transformer)
                .point(0.0, base_value)
                .point(0.1, 0.625)
                .build();
        let ridges_shattered =
            TerrainSplineBuilder::new(TerrainSplineCoordinate::Ridges, transformer)
                .point(-0.9, base_value)
                .spline(-0.69, weirdness_shattered)
                .build();
        builder = builder
            .point(0.35, base_value)
            .spline(0.45, ridges_shattered.clone())
            .spline(0.55, ridges_shattered)
            .point(0.62, base_value);
    } else {
        let hills = TerrainSplineBuilder::new(TerrainSplineCoordinate::Ridges, transformer)
            .spline(-0.7, base.clone())
            .point(-0.15, 1.37)
            .build();
        let peaks = TerrainSplineBuilder::new(TerrainSplineCoordinate::Ridges, transformer)
            .spline(0.45, base)
            .point(0.7, 1.56)
            .build();
        builder = builder
            .spline(0.05, peaks.clone())
            .spline(0.4, peaks)
            .spline(0.45, hills.clone())
            .spline(0.55, hills)
            .point(0.58, base_value);
    }
    builder.build()
}

#[allow(clippy::too_many_arguments)]
fn erosion_offset_spline(
    low_valley: f64,
    hill: f64,
    tall_hill: f64,
    mountain_factor: f64,
    plain: f64,
    swamp: f64,
    include_extreme_hills: bool,
    saddle: bool,
    transformer: fn(f64) -> f64,
) -> TerrainCubicSpline {
    let very_low_mountains =
        mountain_ridge_spline(lerp(mountain_factor, 0.6, 1.5), saddle, transformer);
    let low_mountains = mountain_ridge_spline(lerp(mountain_factor, 0.6, 1.0), saddle, transformer);
    let mountains = mountain_ridge_spline(mountain_factor, saddle, transformer);
    let wide_plateau = ridge_spline(
        low_valley - 0.15,
        0.5 * mountain_factor,
        0.5 * mountain_factor,
        0.5 * mountain_factor,
        0.6 * mountain_factor,
        0.5,
        transformer,
    );
    let narrow_plateau = ridge_spline(
        low_valley,
        plain * mountain_factor,
        hill * mountain_factor,
        0.5 * mountain_factor,
        0.6 * mountain_factor,
        0.5,
        transformer,
    );
    let plains = ridge_spline(low_valley, plain, plain, hill, tall_hill, 0.5, transformer);
    let plains_far = ridge_spline(low_valley, plain, plain, hill, tall_hill, 0.5, transformer);
    let extreme_hills = TerrainSplineBuilder::new(TerrainSplineCoordinate::Ridges, transformer)
        .point(-1.0, low_valley)
        .spline(-0.4, plains.clone())
        .point(0.0, tall_hill + 0.07)
        .build();
    let swamps = ridge_spline(-0.02, swamp, swamp, hill, tall_hill, 0.0, transformer);
    let mut builder = TerrainSplineBuilder::new(TerrainSplineCoordinate::Erosion, transformer)
        .spline(-0.85, very_low_mountains)
        .spline(-0.7, low_mountains)
        .spline(-0.4, mountains)
        .spline(-0.35, wide_plateau)
        .spline(-0.1, narrow_plateau)
        .spline(0.2, plains);
    if include_extreme_hills {
        builder = builder
            .spline(0.4, plains_far.clone())
            .spline(0.45, extreme_hills.clone())
            .spline(0.55, extreme_hills)
            .spline(0.58, plains_far);
    }
    builder.spline(0.7, swamps).build()
}

fn mountain_ridge_spline(
    modulation: f64,
    saddle: bool,
    transformer: fn(f64) -> f64,
) -> TerrainCubicSpline {
    let min = mountain_continentalness(-1.0, modulation);
    let max = mountain_continentalness(1.0, modulation);
    let zero = mountain_ridge_zero_continentalness(modulation);
    let mut builder = TerrainSplineBuilder::new(TerrainSplineCoordinate::Ridges, transformer);
    if (-0.65..1.0).contains(&zero) {
        let after = mountain_continentalness(-0.65, modulation);
        let before = mountain_continentalness(-0.75, modulation);
        let min_derivative = (before - min) / 0.25;
        builder = builder
            .point_derivative(-1.0, min, min_derivative)
            .point(-0.75, before)
            .point(-0.65, after);
        let zero_value = mountain_continentalness(zero, modulation);
        let max_derivative = (max - zero_value) / (1.0 - zero);
        builder = builder
            .point(zero - 0.01, zero_value)
            .point_derivative(zero, zero_value, max_derivative)
            .point_derivative(1.0, max, max_derivative);
    } else {
        let derivative = (max - min) / 2.0;
        if saddle {
            builder = builder.point(-1.0, 0.2_f64.max(min)).point_derivative(
                0.0,
                lerp(0.5, min, max),
                derivative,
            );
        } else {
            builder = builder.point_derivative(-1.0, min, derivative);
        }
        builder = builder.point_derivative(1.0, max, derivative);
    }
    builder.build()
}

fn mountain_continentalness(ridge: f64, modulation: f64) -> f64 {
    let slope = 1.0 - (1.0 - modulation) * 0.5;
    let intersect = 0.5 * (1.0 - modulation);
    let continentalness = (ridge + 1.17) * 0.46082947 * slope - intersect;
    if ridge < -0.7 {
        continentalness.max(-0.2222)
    } else {
        continentalness.max(0.0)
    }
}

fn mountain_ridge_zero_continentalness(modulation: f64) -> f64 {
    let slope = 1.0 - (1.0 - modulation) * 0.5;
    let intersect = 0.5 * (1.0 - modulation);
    intersect / (0.46082947 * slope) - 1.17
}

fn ridge_spline(
    valley: f64,
    low: f64,
    mid: f64,
    high: f64,
    peaks: f64,
    min_valley_steepness: f64,
    transformer: fn(f64) -> f64,
) -> TerrainCubicSpline {
    let d1 = (0.5 * (low - valley)).max(min_valley_steepness);
    let d2 = 5.0 * (mid - low);
    TerrainSplineBuilder::new(TerrainSplineCoordinate::Ridges, transformer)
        .point_derivative(-1.0, valley, d1)
        .point_derivative(-0.4, low, d1.min(d2))
        .point_derivative(0.0, mid, d2)
        .point_derivative(0.4, high, 2.0 * (high - mid))
        .point_derivative(1.0, peaks, 0.7 * (peaks - high))
        .build()
}

