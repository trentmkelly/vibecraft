use super::*;

#[derive(Clone, Copy)]
struct DensityNoiseSampleContext {
    seed: i64,
    settings: NoiseGeneratorSettings,
    block_x: i32,
    block_y: i32,
    block_z: i32,
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
            } => {
                let first = argument1.compute(block_y);
                kind.apply_lazy(first, argument2.value_bounds(), || {
                    argument2.compute(block_y)
                })
            }
            DensityFunction::RangeChoice {
                input,
                min_inclusive,
                max_exclusive,
                when_in_range,
                when_out_of_range,
            } => {
                let input_value = input.compute(block_y);
                if input_value >= min_inclusive && input_value < max_exclusive {
                    when_in_range.compute(block_y)
                } else {
                    when_out_of_range.compute(block_y)
                }
            }
            DensityFunction::Marker { input, .. } | DensityFunction::BlendDensity { input } => {
                input.compute(block_y)
            }
            DensityFunction::BlendAlpha => 1.0,
            DensityFunction::BlendOffset => 0.0,
            DensityFunction::Noise { .. }
            | DensityFunction::ShiftA { .. }
            | DensityFunction::ShiftB { .. }
            | DensityFunction::Shift { .. }
            | DensityFunction::ShiftedNoise { .. }
            | DensityFunction::BlendedNoise { .. }
            | DensityFunction::EndIslands { .. }
            | DensityFunction::WeirdScaledSampler { .. }
            | DensityFunction::Beardifier
            | DensityFunction::Spline { .. } => 0.0,
            DensityFunction::FindTopSurface {
                density,
                upper_bound,
                lower_bound,
                cell_height,
            } => find_top_surface_compute(
                |block_y| density.compute(block_y),
                upper_bound.compute(block_y),
                lower_bound,
                cell_height,
            ),
        }
    }

    pub fn compute_with_noise(
        self,
        seed: i64,
        settings: NoiseGeneratorSettings,
        block_x: i32,
        block_y: i32,
        block_z: i32,
    ) -> f64 {
        self.compute_with_noise_context(DensityNoiseSampleContext {
            seed,
            settings,
            block_x,
            block_y,
            block_z,
        })
    }

    fn compute_with_noise_context(self, context: DensityNoiseSampleContext) -> f64 {
        match self {
            DensityFunction::Reference(_)
            | DensityFunction::Constant(_)
            | DensityFunction::YClampedGradient { .. }
            | DensityFunction::Clamp { .. }
            | DensityFunction::Mapped { .. }
            | DensityFunction::Binary { .. }
            | DensityFunction::RangeChoice { .. }
            | DensityFunction::Marker { .. }
            | DensityFunction::BlendDensity { .. } => {
                self.compute_composed_density_with_noise(context)
            }
            DensityFunction::Noise { .. }
            | DensityFunction::ShiftA { .. }
            | DensityFunction::ShiftB { .. }
            | DensityFunction::Shift { .. }
            | DensityFunction::ShiftedNoise { .. }
            | DensityFunction::WeirdScaledSampler { .. } => {
                self.compute_sampled_density_with_noise(context)
            }
            DensityFunction::BlendAlpha
            | DensityFunction::BlendOffset
            | DensityFunction::BlendedNoise { .. }
            | DensityFunction::EndIslands { .. }
            | DensityFunction::Beardifier
            | DensityFunction::Spline { .. }
            | DensityFunction::FindTopSurface { .. } => {
                self.compute_special_density_with_noise(context)
            }
        }
    }

    fn compute_composed_density_with_noise(self, context: DensityNoiseSampleContext) -> f64 {
        match self {
            DensityFunction::Reference(id) => compute_density_reference_with_noise(id, context),
            DensityFunction::Constant(value) => value,
            DensityFunction::YClampedGradient { .. } => self.compute(context.block_y),
            DensityFunction::Clamp { input, min, max } => {
                input.compute_with_noise_context(context).clamp(min, max)
            }
            DensityFunction::Mapped { kind, input } => {
                kind.transform(input.compute_with_noise_context(context))
            }
            DensityFunction::Binary {
                kind,
                argument1,
                argument2,
            } => compute_binary_density_with_noise(kind, argument1, argument2, context),
            DensityFunction::RangeChoice {
                input,
                min_inclusive,
                max_exclusive,
                when_in_range,
                when_out_of_range,
            } => compute_range_choice_density_with_noise(
                input,
                min_inclusive,
                max_exclusive,
                when_in_range,
                when_out_of_range,
                context,
            ),
            DensityFunction::Marker { input, .. } | DensityFunction::BlendDensity { input } => {
                input.compute_with_noise_context(context)
            }
            _ => unreachable!("composed density dispatch received a non-composed function"),
        }
    }

    fn compute_sampled_density_with_noise(self, context: DensityNoiseSampleContext) -> f64 {
        match self {
            DensityFunction::Noise {
                noise,
                xz_scale,
                y_scale,
            } => random_state_normal_noise_sample(
                context.seed,
                context.settings,
                noise,
                f64::from(context.block_x) * xz_scale,
                f64::from(context.block_y) * y_scale,
                f64::from(context.block_z) * xz_scale,
            ),
            DensityFunction::ShiftA { noise } => density_shift_noise_sample(
                context.seed,
                context.settings,
                noise,
                f64::from(context.block_x),
                0.0,
                f64::from(context.block_z),
            ),
            DensityFunction::ShiftB { noise } => density_shift_noise_sample(
                context.seed,
                context.settings,
                noise,
                f64::from(context.block_z),
                f64::from(context.block_x),
                0.0,
            ),
            DensityFunction::Shift { noise } => density_shift_noise_sample(
                context.seed,
                context.settings,
                noise,
                f64::from(context.block_x),
                f64::from(context.block_y),
                f64::from(context.block_z),
            ),
            DensityFunction::ShiftedNoise {
                shift_x,
                shift_y,
                shift_z,
                xz_scale,
                y_scale,
                noise,
            } => {
                let x = f64::from(context.block_x) * xz_scale
                    + shift_x.compute_with_noise_context(context);
                let y = f64::from(context.block_y) * y_scale
                    + shift_y.compute_with_noise_context(context);
                let z = f64::from(context.block_z) * xz_scale
                    + shift_z.compute_with_noise_context(context);
                random_state_normal_noise_sample(context.seed, context.settings, noise, x, y, z)
            }
            DensityFunction::WeirdScaledSampler {
                input,
                noise,
                rarity_mapper,
            } => {
                let rarity = rarity_mapper.map_value(input.compute_with_noise_context(context));
                rarity
                    * random_state_normal_noise_sample(
                        context.seed,
                        context.settings,
                        noise,
                        f64::from(context.block_x) / rarity,
                        f64::from(context.block_y) / rarity,
                        f64::from(context.block_z) / rarity,
                    )
                    .abs()
            }
            _ => unreachable!("sampled density dispatch received a non-sampled function"),
        }
    }

    fn compute_special_density_with_noise(self, context: DensityNoiseSampleContext) -> f64 {
        match self {
            DensityFunction::BlendAlpha => 1.0,
            DensityFunction::BlendOffset => 0.0,
            DensityFunction::BlendedNoise {
                xz_scale,
                y_scale,
                xz_factor,
                y_factor,
                smear_scale_multiplier,
            } => blended_noise_snapshot(
                random_state_terrain_random(context.seed, context.settings),
                xz_scale,
                y_scale,
                xz_factor,
                y_factor,
                smear_scale_multiplier,
            )
            .map(|snapshot| {
                blended_noise_sample(
                    &snapshot,
                    f64::from(context.block_x),
                    f64::from(context.block_y),
                    f64::from(context.block_z),
                )
            })
            .unwrap_or(0.0),
            DensityFunction::EndIslands {
                seed: function_seed,
            } => end_island_density_sample(
                if function_seed == 0 {
                    context.seed
                } else {
                    function_seed
                },
                context.block_x,
                context.block_z,
            ),
            DensityFunction::Beardifier => 0.0,
            DensityFunction::Spline { kind } => terrain_spline(kind).apply(terrain_spline_context(
                context.seed,
                context.settings,
                context.block_x,
                context.block_y,
                context.block_z,
            )),
            DensityFunction::FindTopSurface {
                density,
                upper_bound,
                lower_bound,
                cell_height,
            } => find_top_surface_compute(
                |sample_y| {
                    density.compute_with_noise_context(DensityNoiseSampleContext {
                        block_y: sample_y,
                        ..context
                    })
                },
                upper_bound.compute_with_noise_context(context),
                lower_bound,
                cell_height,
            ),
            _ => unreachable!("special density dispatch received a non-special function"),
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
            DensityFunction::RangeChoice { .. } => "range_choice",
            DensityFunction::Marker { kind, .. } => kind.serialized_name(),
            DensityFunction::Noise { .. } => "noise",
            DensityFunction::ShiftA { .. } => "shift_a",
            DensityFunction::ShiftB { .. } => "shift_b",
            DensityFunction::Shift { .. } => "shift",
            DensityFunction::ShiftedNoise { .. } => "shifted_noise",
            DensityFunction::BlendedNoise { .. } => "old_blended_noise",
            DensityFunction::EndIslands { .. } => "end_islands",
            DensityFunction::WeirdScaledSampler { .. } => "weird_scaled_sampler",
            DensityFunction::BlendAlpha => "blend_alpha",
            DensityFunction::BlendOffset => "blend_offset",
            DensityFunction::BlendDensity { .. } => "blend_density",
            DensityFunction::Beardifier => "beardifier",
            DensityFunction::Spline { .. } => "spline",
            DensityFunction::FindTopSurface { .. } => "find_top_surface",
        }
    }

    pub fn value_bounds(self) -> (f64, f64) {
        match self {
            DensityFunction::Reference(id) => builtin_density_function(id)
                .map(|entry| entry.function.value_bounds())
                .unwrap_or((f64::NEG_INFINITY, f64::INFINITY)),
            DensityFunction::Constant(value) => (value, value),
            DensityFunction::YClampedGradient {
                from_value,
                to_value,
                ..
            } => (from_value.min(to_value), from_value.max(to_value)),
            DensityFunction::Clamp { min, max, .. } => (min, max),
            DensityFunction::Mapped { kind, input } => kind.value_bounds(input.value_bounds()),
            DensityFunction::Binary {
                kind,
                argument1,
                argument2,
            } => kind.value_bounds(argument1.value_bounds(), argument2.value_bounds()),
            DensityFunction::RangeChoice {
                when_in_range,
                when_out_of_range,
                ..
            } => {
                let in_range = when_in_range.value_bounds();
                let out_of_range = when_out_of_range.value_bounds();
                (
                    in_range.0.min(out_of_range.0),
                    in_range.1.max(out_of_range.1),
                )
            }
            DensityFunction::Marker { input, .. } => input.value_bounds(),
            DensityFunction::BlendDensity { .. } => (f64::NEG_INFINITY, f64::INFINITY),
            DensityFunction::BlendAlpha => (1.0, 1.0),
            DensityFunction::BlendOffset | DensityFunction::Beardifier => (0.0, 0.0),
            DensityFunction::EndIslands { .. } => (-0.84375, 0.5625),
            DensityFunction::FindTopSurface {
                upper_bound,
                lower_bound,
                ..
            } => {
                let upper = upper_bound.value_bounds();
                (f64::from(lower_bound), f64::from(lower_bound).max(upper.1))
            }
            DensityFunction::WeirdScaledSampler {
                noise,
                rarity_mapper,
                ..
            } => builtin_normal_noise_parameters(noise)
                .map(|parameters| {
                    (
                        0.0,
                        rarity_mapper.max_rarity() * normal_noise_max_value(*parameters),
                    )
                })
                .unwrap_or((0.0, f64::INFINITY)),
            DensityFunction::Noise { noise, .. } | DensityFunction::ShiftedNoise { noise, .. } => {
                normal_noise_value_bounds(noise).unwrap_or((f64::NEG_INFINITY, f64::INFINITY))
            }
            DensityFunction::ShiftA { noise }
            | DensityFunction::ShiftB { noise }
            | DensityFunction::Shift { noise } => normal_noise_value_bounds(noise)
                .map(|(min, max)| (min * 4.0, max * 4.0))
                .unwrap_or((f64::NEG_INFINITY, f64::INFINITY)),
            DensityFunction::BlendedNoise { y_scale, .. } => {
                let max_value = blended_noise_max_value(y_scale);
                (-max_value, max_value)
            }
            DensityFunction::Spline { kind } => terrain_spline(kind).bounds(),
        }
    }
}

fn compute_density_reference_with_noise(
    id: &'static str,
    context: DensityNoiseSampleContext,
) -> f64 {
    builtin_density_function(id)
        .map(|entry| entry.function.compute_with_noise_context(context))
        .unwrap_or(0.0)
}

fn compute_binary_density_with_noise(
    kind: BinaryDensityFunction,
    argument1: &'static DensityFunction,
    argument2: &'static DensityFunction,
    context: DensityNoiseSampleContext,
) -> f64 {
    let first = argument1.compute_with_noise_context(context);
    kind.apply_lazy(first, argument2.value_bounds(), || {
        argument2.compute_with_noise_context(context)
    })
}

fn compute_range_choice_density_with_noise(
    input: &'static DensityFunction,
    min_inclusive: f64,
    max_exclusive: f64,
    when_in_range: &'static DensityFunction,
    when_out_of_range: &'static DensityFunction,
    context: DensityNoiseSampleContext,
) -> f64 {
    let input_value = input.compute_with_noise_context(context);
    if input_value >= min_inclusive && input_value < max_exclusive {
        when_in_range.compute_with_noise_context(context)
    } else {
        when_out_of_range.compute_with_noise_context(context)
    }
}

pub fn find_top_surface_compute(
    mut density_at_y: impl FnMut(i32) -> f64,
    upper_bound: f64,
    lower_bound: i32,
    cell_height: i32,
) -> f64 {
    if cell_height <= 0 {
        return f64::from(lower_bound);
    }
    let top_y = (upper_bound.floor() as i32).div_euclid(cell_height) * cell_height;
    if top_y <= lower_bound {
        return f64::from(lower_bound);
    }
    let mut block_y = top_y;
    while block_y >= lower_bound {
        if density_at_y(block_y) > 0.0 {
            return f64::from(block_y);
        }
        block_y -= cell_height;
    }
    f64::from(lower_bound)
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
            MappedDensityFunction::Invert => 1.0 / input,
            MappedDensityFunction::Squeeze => {
                let clamped = input.clamp(-1.0, 1.0);
                clamped / 2.0 - clamped * clamped * clamped / 24.0
            }
        }
    }

    pub fn value_bounds(self, input: (f64, f64)) -> (f64, f64) {
        match self {
            MappedDensityFunction::Abs => {
                if input.0 >= 0.0 {
                    input
                } else if input.1 <= 0.0 {
                    (-input.1, -input.0)
                } else {
                    (0.0, input.0.abs().max(input.1.abs()))
                }
            }
            MappedDensityFunction::Square => {
                if input.0 >= 0.0 {
                    (input.0 * input.0, input.1 * input.1)
                } else if input.1 <= 0.0 {
                    (input.1 * input.1, input.0 * input.0)
                } else {
                    (0.0, input.0.abs().max(input.1.abs()).powi(2))
                }
            }
            MappedDensityFunction::Cube => (input.0.powi(3), input.1.powi(3)),
            MappedDensityFunction::HalfNegative => {
                (self.transform(input.0), self.transform(input.1))
            }
            MappedDensityFunction::QuarterNegative => {
                (self.transform(input.0), self.transform(input.1))
            }
            MappedDensityFunction::Invert => (-input.1, -input.0),
            MappedDensityFunction::Squeeze => {
                let min = self.transform(input.0);
                let max = self.transform(input.1);
                (min.min(max), min.max(max))
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

    pub fn apply_lazy(
        self,
        first: f64,
        second_bounds: (f64, f64),
        second: impl FnOnce() -> f64,
    ) -> f64 {
        match self {
            BinaryDensityFunction::Add => first + second(),
            BinaryDensityFunction::Mul => {
                if first == 0.0 {
                    0.0
                } else {
                    first * second()
                }
            }
            BinaryDensityFunction::Min => {
                if first < second_bounds.0 {
                    first
                } else {
                    first.min(second())
                }
            }
            BinaryDensityFunction::Max => {
                if first > second_bounds.1 {
                    first
                } else {
                    first.max(second())
                }
            }
        }
    }

    pub fn value_bounds(self, first: (f64, f64), second: (f64, f64)) -> (f64, f64) {
        match self {
            BinaryDensityFunction::Add => (first.0 + second.0, first.1 + second.1),
            BinaryDensityFunction::Mul => {
                let products = [
                    first.0 * second.0,
                    first.0 * second.1,
                    first.1 * second.0,
                    first.1 * second.1,
                ];
                products
                    .into_iter()
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), value| {
                        (min.min(value), max.max(value))
                    })
            }
            BinaryDensityFunction::Min => (first.0.min(second.0), first.1.min(second.1)),
            BinaryDensityFunction::Max => (first.0.max(second.0), first.1.max(second.1)),
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

    pub fn map_value(self, rarity_factor: f64) -> f64 {
        match self {
            RarityValueMapper::Type1 => {
                if rarity_factor < -0.5 {
                    0.75
                } else if rarity_factor < 0.0 {
                    1.0
                } else if rarity_factor < 0.5 {
                    1.5
                } else {
                    2.0
                }
            }
            RarityValueMapper::Type2 => {
                if rarity_factor < -0.75 {
                    0.5
                } else if rarity_factor < -0.5 {
                    0.75
                } else if rarity_factor < 0.5 {
                    1.0
                } else if rarity_factor < 0.75 {
                    2.0
                } else {
                    3.0
                }
            }
        }
    }

    pub fn max_rarity(self) -> f64 {
        match self {
            RarityValueMapper::Type1 => 2.0,
            RarityValueMapper::Type2 => 3.0,
        }
    }
}
