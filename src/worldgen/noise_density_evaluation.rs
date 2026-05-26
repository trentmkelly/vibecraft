use super::*;

#[derive(Debug, Clone)]
pub(super) struct CacheOnceState {
    last_counter: i64,
    last_array_counter: i64,
    last_value: f64,
    last_array: Option<Vec<f64>>,
}

impl Default for CacheOnceState {
    fn default() -> Self {
        Self {
            last_counter: i64::MIN,
            last_array_counter: i64::MIN,
            last_value: 0.0,
            last_array: None,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum DensityArrayFillMode {
    Slice { block_x: i32, block_z: i32 },
    Cell,
}

/// Evaluate a `DensityFunction` tree, substituting interpolated values for
/// every `Marker(Interpolated)` node whose inner function is tracked by the
/// chunk's interpolator table.
///
/// Takes `&NoiseChunk` directly (instead of a pre-built HashMap) to avoid a
/// 98,304-allocation-per-chunk overhead and to correctly handle
/// `DensityFunction::Reference` nodes by following them recursively.
///
/// This mirrors how Java's `NoiseChunk` wraps each density function so that
/// `compute(NoiseChunk.this)` returns the pre-interpolated value.
pub(super) fn eval_density_fn_with_interp(
    df: DensityFunction,
    chunk: &NoiseChunk,
    x: i32,
    y: i32,
    z: i32,
) -> f64 {
    let pos = BlockPos { x, y, z };
    match df {
        DensityFunction::Marker { kind, input } => {
            eval_density_marker_with_interp(kind, input, chunk, pos)
        }
        // Follow references into the built-in density function registry.
        // This is critical: the overworld's final_density IS a Reference node,
        // so without this arm the entire interpolation system is bypassed.
        DensityFunction::Reference(_)
        | DensityFunction::BlendDensity { .. }
        | DensityFunction::Clamp { .. }
        | DensityFunction::Mapped { .. }
        | DensityFunction::Binary { .. }
        | DensityFunction::RangeChoice { .. }
        | DensityFunction::Spline { .. } => eval_density_composed_with_interp(df, chunk, pos),
        // All remaining variants have no children containing Interpolated markers
        // (or are already fully evaluated at slice-fill time): delegate to
        // compute_with_noise for correctness.
        other => eval_density_terminal_with_interp(other, chunk, pos),
    }
}

fn eval_density_marker_with_interp(
    kind: DensityMarker,
    input: &'static DensityFunction,
    chunk: &NoiseChunk,
    pos: BlockPos,
) -> f64 {
    match kind {
        DensityMarker::Interpolated => eval_interpolated_marker(input, chunk, pos),
        DensityMarker::Cache2D => eval_cache_2d_marker(input, chunk, pos),
        DensityMarker::CacheOnce => eval_cache_once_marker(input, chunk, pos),
        DensityMarker::FlatCache => eval_flat_cache_marker(input, chunk, pos),
        DensityMarker::CacheAllInCell => eval_cache_all_in_cell_marker(input, chunk, pos),
    }
}

fn eval_interpolated_marker(
    input: &'static DensityFunction,
    chunk: &NoiseChunk,
    pos: BlockPos,
) -> f64 {
    let ptr = input as *const DensityFunction as usize;
    if let Some(idx) = lookup_density_index(&chunk.interp_by_ptr, ptr) {
        chunk.interpolator_value(idx)
    } else {
        input.compute_with_noise(chunk.seed, chunk.settings, pos.x, pos.y, pos.z)
    }
}

fn eval_cache_2d_marker(input: &'static DensityFunction, chunk: &NoiseChunk, pos: BlockPos) -> f64 {
    let ptr = input as *const DensityFunction as usize;
    let pos_key = pack_column(pos.x, pos.z);
    let Some(cache_index) = lookup_density_index(&chunk.cache_2d_by_ptr, ptr) else {
        return eval_density_fn_with_interp(*input, chunk, pos.x, pos.y, pos.z);
    };
    if let Some(value) = {
        let cache = chunk.cache_2d.borrow();
        let state = &cache[cache_index];
        (state.last_pos_2d == pos_key).then_some(state.last_value)
    } {
        return value;
    }

    let inner_fn = {
        let cache = chunk.cache_2d.borrow();
        cache[cache_index].inner_fn
    };
    let value = eval_density_fn_with_interp(*inner_fn, chunk, pos.x, pos.y, pos.z);
    let mut cache = chunk.cache_2d.borrow_mut();
    let state = &mut cache[cache_index];
    state.last_pos_2d = pos_key;
    state.last_value = value;
    value
}

fn eval_cache_once_marker(
    input: &'static DensityFunction,
    chunk: &NoiseChunk,
    pos: BlockPos,
) -> f64 {
    let ptr = input as *const DensityFunction as usize;
    let Some(cache_index) = lookup_density_index(&chunk.cache_once_by_ptr, ptr) else {
        chunk
            .cache_once_scalar_misses
            .set(chunk.cache_once_scalar_misses.get().saturating_add(1));
        return eval_density_fn_with_interp(*input, chunk, pos.x, pos.y, pos.z);
    };
    if let Some(value) = cache_once_scalar_value(chunk, cache_index) {
        chunk
            .cache_once_scalar_hits
            .set(chunk.cache_once_scalar_hits.get().saturating_add(1));
        return value;
    }
    chunk
        .cache_once_scalar_misses
        .set(chunk.cache_once_scalar_misses.get().saturating_add(1));
    let value = eval_density_fn_with_interp(*input, chunk, pos.x, pos.y, pos.z);
    let mut cache = chunk.cache_once_values.borrow_mut();
    let state = &mut cache[cache_index];
    state.last_counter = chunk.interpolation_counter;
    state.last_value = value;
    value
}

fn cache_once_scalar_value(chunk: &NoiseChunk, cache_index: usize) -> Option<f64> {
    let cache = chunk.cache_once_values.borrow();
    let state = &cache[cache_index];
    if state.last_array_counter == chunk.array_interpolation_counter {
        state
            .last_array
            .as_ref()
            .and_then(|array| array.get(chunk.array_index).copied())
    } else if state.last_counter == chunk.interpolation_counter {
        Some(state.last_value)
    } else {
        None
    }
}

fn eval_flat_cache_marker(
    input: &'static DensityFunction,
    chunk: &NoiseChunk,
    pos: BlockPos,
) -> f64 {
    let ptr = input as *const DensityFunction as usize;
    if let Some(cache_index) = lookup_density_index(&chunk.flat_cache_by_ptr, ptr) {
        let cache = &chunk.flat_cache[cache_index];
        let quart_x = pos.x >> 2;
        let quart_z = pos.z >> 2;
        let local_x = quart_x - ((chunk.first_cell_x * chunk.cell_width) >> 2);
        let local_z = quart_z - ((chunk.first_cell_z * chunk.cell_width) >> 2);
        if local_x >= 0
            && local_z >= 0
            && (local_x as usize) < cache.size_xz
            && (local_z as usize) < cache.size_xz
        {
            return cache.values[local_x as usize + local_z as usize * cache.size_xz];
        }
        return eval_density_fn_with_interp(*cache.inner_fn, chunk, pos.x, pos.y, pos.z);
    }
    eval_density_fn_with_interp(*input, chunk, pos.x, pos.y, pos.z)
}

fn eval_cache_all_in_cell_marker(
    input: &'static DensityFunction,
    chunk: &NoiseChunk,
    pos: BlockPos,
) -> f64 {
    if !chunk.filling_cell_cache.get() {
        let ptr = input as *const DensityFunction as usize;
        if let (Some(cache_index), Some(value_index)) = (
            lookup_density_index(&chunk.cache_all_by_ptr, ptr),
            chunk.cache_all_cell_index(),
        ) {
            if let Some(value) = chunk.cache_all_in_cell[cache_index].values.get(value_index) {
                return *value;
            }
        }
    }
    eval_density_fn_with_interp(*input, chunk, pos.x, pos.y, pos.z)
}

fn eval_density_composed_with_interp(
    df: DensityFunction,
    chunk: &NoiseChunk,
    pos: BlockPos,
) -> f64 {
    match df {
        DensityFunction::Reference(id) => builtin_density_function(id)
            .map(|entry| eval_density_fn_with_interp(entry.function, chunk, pos.x, pos.y, pos.z))
            .unwrap_or(0.0),
        DensityFunction::BlendDensity { input } => {
            eval_density_fn_with_interp(*input, chunk, pos.x, pos.y, pos.z)
        }
        DensityFunction::Clamp { input, min, max } => {
            eval_density_fn_with_interp(*input, chunk, pos.x, pos.y, pos.z).clamp(min, max)
        }
        DensityFunction::Mapped { kind, input } => kind.transform(eval_density_fn_with_interp(
            *input, chunk, pos.x, pos.y, pos.z,
        )),
        DensityFunction::Binary {
            kind,
            argument1,
            argument2,
        } => eval_binary_density_with_interp(kind, argument1, argument2, chunk, pos),
        DensityFunction::RangeChoice {
            input,
            min_inclusive,
            max_exclusive,
            when_in_range,
            when_out_of_range,
        } => eval_range_choice_with_interp(
            input,
            min_inclusive,
            max_exclusive,
            when_in_range,
            when_out_of_range,
            chunk,
            pos,
        ),
        DensityFunction::Spline { kind } => chunk.terrain_spline_value(
            kind,
            terrain_spline_context(chunk.seed, chunk.settings, pos.x, pos.y, pos.z),
        ),
        _ => unreachable!("non-composed density function routed to composed evaluator"),
    }
}

fn eval_binary_density_with_interp(
    kind: BinaryDensityFunction,
    argument1: &'static DensityFunction,
    argument2: &'static DensityFunction,
    chunk: &NoiseChunk,
    pos: BlockPos,
) -> f64 {
    let first = eval_density_fn_with_interp(*argument1, chunk, pos.x, pos.y, pos.z);
    let second_bounds = chunk.density_value_bounds(argument2);
    kind.apply_lazy(first, second_bounds, || {
        eval_density_fn_with_interp(*argument2, chunk, pos.x, pos.y, pos.z)
    })
}

fn eval_range_choice_with_interp(
    input: &'static DensityFunction,
    min_inclusive: f64,
    max_exclusive: f64,
    when_in_range: &'static DensityFunction,
    when_out_of_range: &'static DensityFunction,
    chunk: &NoiseChunk,
    pos: BlockPos,
) -> f64 {
    let value = eval_density_fn_with_interp(*input, chunk, pos.x, pos.y, pos.z);
    if value >= min_inclusive && value < max_exclusive {
        eval_density_fn_with_interp(*when_in_range, chunk, pos.x, pos.y, pos.z)
    } else {
        eval_density_fn_with_interp(*when_out_of_range, chunk, pos.x, pos.y, pos.z)
    }
}

fn eval_density_terminal_with_interp(
    df: DensityFunction,
    chunk: &NoiseChunk,
    pos: BlockPos,
) -> f64 {
    match df {
        DensityFunction::Noise {
            noise,
            xz_scale,
            y_scale,
        } => chunk.normal_noise_sample(
            noise,
            f64::from(pos.x) * xz_scale,
            f64::from(pos.y) * y_scale,
            f64::from(pos.z) * xz_scale,
        ),
        DensityFunction::ShiftA { noise } => {
            chunk.normal_noise_sample(noise, f64::from(pos.x) * 0.25, 0.0, f64::from(pos.z) * 0.25)
                * 4.0
        }
        DensityFunction::ShiftB { noise } => {
            chunk.normal_noise_sample(noise, f64::from(pos.z) * 0.25, f64::from(pos.x) * 0.25, 0.0)
                * 4.0
        }
        DensityFunction::Shift { noise } => {
            chunk.normal_noise_sample(
                noise,
                f64::from(pos.x) * 0.25,
                f64::from(pos.y) * 0.25,
                f64::from(pos.z) * 0.25,
            ) * 4.0
        }
        DensityFunction::ShiftedNoise {
            shift_x,
            shift_y,
            shift_z,
            xz_scale,
            y_scale,
            noise,
        } => eval_shifted_noise_with_interp(
            ShiftedNoiseEval {
                shift_x,
                shift_y,
                shift_z,
                xz_scale,
                y_scale,
                noise,
            },
            chunk,
            pos,
        ),
        DensityFunction::BlendedNoise {
            xz_scale,
            y_scale,
            xz_factor,
            y_factor,
            smear_scale_multiplier,
        } => chunk.blended_noise_sample(
            xz_scale,
            y_scale,
            xz_factor,
            y_factor,
            smear_scale_multiplier,
            f64::from(pos.x),
            f64::from(pos.y),
            f64::from(pos.z),
        ),
        DensityFunction::WeirdScaledSampler {
            input,
            noise,
            rarity_mapper,
        } => {
            let input_value = eval_density_fn_with_interp(*input, chunk, pos.x, pos.y, pos.z);
            weird_scaled_sampler_value(
                chunk,
                noise,
                rarity_mapper,
                pos.x,
                pos.y,
                pos.z,
                input_value,
            )
        }
        other => other.compute_with_noise(chunk.seed, chunk.settings, pos.x, pos.y, pos.z),
    }
}

#[derive(Clone, Copy)]
struct ShiftedNoiseEval {
    shift_x: &'static DensityFunction,
    shift_y: &'static DensityFunction,
    shift_z: &'static DensityFunction,
    xz_scale: f64,
    y_scale: f64,
    noise: &'static str,
}

fn eval_shifted_noise_with_interp(
    shifted: ShiftedNoiseEval,
    chunk: &NoiseChunk,
    pos: BlockPos,
) -> f64 {
    let sample_x = f64::from(pos.x) * shifted.xz_scale
        + eval_density_fn_with_interp(*shifted.shift_x, chunk, pos.x, pos.y, pos.z);
    let sample_y = f64::from(pos.y) * shifted.y_scale
        + eval_density_fn_with_interp(*shifted.shift_y, chunk, pos.x, pos.y, pos.z);
    let sample_z = f64::from(pos.z) * shifted.xz_scale
        + eval_density_fn_with_interp(*shifted.shift_z, chunk, pos.x, pos.y, pos.z);
    chunk.normal_noise_sample(shifted.noise, sample_x, sample_y, sample_z)
}

fn weird_scaled_sampler_value(
    chunk: &NoiseChunk,
    noise: &'static str,
    rarity_mapper: RarityValueMapper,
    block_x: i32,
    block_y: i32,
    block_z: i32,
    input_value: f64,
) -> f64 {
    let rarity = rarity_mapper.map_value(input_value);
    rarity
        * chunk
            .normal_noise_sample(
                noise,
                f64::from(block_x) / rarity,
                f64::from(block_y) / rarity,
                f64::from(block_z) / rarity,
            )
            .abs()
}

pub(super) fn eval_density_fn_single_point(
    df: DensityFunction,
    chunk: &NoiseChunk,
    x: i32,
    y: i32,
    z: i32,
) -> f64 {
    let pos = BlockPos { x, y, z };
    match df {
        DensityFunction::Reference(_) | DensityFunction::Marker { .. } => {
            eval_density_single_reference_or_marker(df, chunk, pos)
        }
        DensityFunction::BlendDensity { .. }
        | DensityFunction::Clamp { .. }
        | DensityFunction::Mapped { .. }
        | DensityFunction::Binary { .. }
        | DensityFunction::RangeChoice { .. }
        | DensityFunction::Spline { .. } => eval_density_single_composed(df, chunk, pos),
        other => eval_density_single_terminal(other, chunk, pos),
    }
}

fn eval_density_single_reference_or_marker(
    df: DensityFunction,
    chunk: &NoiseChunk,
    pos: BlockPos,
) -> f64 {
    match df {
        DensityFunction::Reference(id) => builtin_density_function(id)
            .map(|entry| eval_density_fn_single_point(entry.function, chunk, pos.x, pos.y, pos.z))
            .unwrap_or(0.0),
        DensityFunction::Marker { kind, input } => match kind {
            DensityMarker::FlatCache => eval_single_flat_cache_marker(input, chunk, pos),
            DensityMarker::Cache2D => eval_single_cache_2d_marker(input, chunk, pos),
            DensityMarker::Interpolated
            | DensityMarker::CacheOnce
            | DensityMarker::CacheAllInCell => {
                eval_density_fn_single_point(*input, chunk, pos.x, pos.y, pos.z)
            }
        },
        _ => unreachable!("non-reference marker routed to single marker evaluator"),
    }
}

fn eval_single_flat_cache_marker(
    input: &'static DensityFunction,
    chunk: &NoiseChunk,
    pos: BlockPos,
) -> f64 {
    let ptr = input as *const DensityFunction as usize;
    if let Some(cache_index) = lookup_density_index(&chunk.flat_cache_by_ptr, ptr) {
        let cache = &chunk.flat_cache[cache_index];
        let quart_x = pos.x >> 2;
        let quart_z = pos.z >> 2;
        let local_x = quart_x - ((chunk.first_cell_x * chunk.cell_width) >> 2);
        let local_z = quart_z - ((chunk.first_cell_z * chunk.cell_width) >> 2);
        if local_x >= 0
            && local_z >= 0
            && (local_x as usize) < cache.size_xz
            && (local_z as usize) < cache.size_xz
        {
            return cache.values[local_x as usize + local_z as usize * cache.size_xz];
        }
    }
    eval_density_fn_single_point(*input, chunk, pos.x, pos.y, pos.z)
}

fn eval_single_cache_2d_marker(
    input: &'static DensityFunction,
    chunk: &NoiseChunk,
    pos: BlockPos,
) -> f64 {
    let ptr = input as *const DensityFunction as usize;
    let pos_key = pack_column(pos.x, pos.z);
    let Some(cache_index) = lookup_density_index(&chunk.cache_2d_by_ptr, ptr) else {
        return eval_density_fn_single_point(*input, chunk, pos.x, pos.y, pos.z);
    };
    if let Some(value) = {
        let cache = chunk.cache_2d.borrow();
        let state = &cache[cache_index];
        (state.last_pos_2d == pos_key).then_some(state.last_value)
    } {
        return value;
    }

    let inner_fn = {
        let cache = chunk.cache_2d.borrow();
        cache[cache_index].inner_fn
    };
    let value = eval_density_fn_single_point(*inner_fn, chunk, pos.x, pos.y, pos.z);
    let mut cache = chunk.cache_2d.borrow_mut();
    let state = &mut cache[cache_index];
    state.last_pos_2d = pos_key;
    state.last_value = value;
    value
}

fn eval_density_single_composed(df: DensityFunction, chunk: &NoiseChunk, pos: BlockPos) -> f64 {
    match df {
        DensityFunction::BlendDensity { input } => {
            eval_density_fn_single_point(*input, chunk, pos.x, pos.y, pos.z)
        }
        DensityFunction::Clamp { input, min, max } => {
            eval_density_fn_single_point(*input, chunk, pos.x, pos.y, pos.z).clamp(min, max)
        }
        DensityFunction::Mapped { kind, input } => kind.transform(eval_density_fn_single_point(
            *input, chunk, pos.x, pos.y, pos.z,
        )),
        DensityFunction::Binary {
            kind,
            argument1,
            argument2,
        } => eval_single_binary_density(kind, argument1, argument2, chunk, pos),
        DensityFunction::RangeChoice {
            input,
            min_inclusive,
            max_exclusive,
            when_in_range,
            when_out_of_range,
        } => eval_single_range_choice(
            input,
            min_inclusive,
            max_exclusive,
            when_in_range,
            when_out_of_range,
            chunk,
            pos,
        ),
        DensityFunction::Spline { kind } => chunk.terrain_spline_value(
            kind,
            terrain_spline_context(chunk.seed, chunk.settings, pos.x, pos.y, pos.z),
        ),
        _ => unreachable!("non-composed density function routed to single composed evaluator"),
    }
}

fn eval_single_binary_density(
    kind: BinaryDensityFunction,
    argument1: &'static DensityFunction,
    argument2: &'static DensityFunction,
    chunk: &NoiseChunk,
    pos: BlockPos,
) -> f64 {
    let first = eval_density_fn_single_point(*argument1, chunk, pos.x, pos.y, pos.z);
    let second_bounds = chunk.density_value_bounds(argument2);
    kind.apply_lazy(first, second_bounds, || {
        eval_density_fn_single_point(*argument2, chunk, pos.x, pos.y, pos.z)
    })
}

fn eval_single_range_choice(
    input: &'static DensityFunction,
    min_inclusive: f64,
    max_exclusive: f64,
    when_in_range: &'static DensityFunction,
    when_out_of_range: &'static DensityFunction,
    chunk: &NoiseChunk,
    pos: BlockPos,
) -> f64 {
    let value = eval_density_fn_single_point(*input, chunk, pos.x, pos.y, pos.z);
    if value >= min_inclusive && value < max_exclusive {
        eval_density_fn_single_point(*when_in_range, chunk, pos.x, pos.y, pos.z)
    } else {
        eval_density_fn_single_point(*when_out_of_range, chunk, pos.x, pos.y, pos.z)
    }
}

fn eval_density_single_terminal(df: DensityFunction, chunk: &NoiseChunk, pos: BlockPos) -> f64 {
    match df {
        DensityFunction::Noise { .. }
        | DensityFunction::ShiftA { .. }
        | DensityFunction::ShiftB { .. }
        | DensityFunction::Shift { .. }
        | DensityFunction::ShiftedNoise { .. }
        | DensityFunction::BlendedNoise { .. }
        | DensityFunction::WeirdScaledSampler { .. } => {
            eval_density_single_noise_terminal(df, chunk, pos)
        }
        DensityFunction::FindTopSurface { .. }
        | DensityFunction::Constant(_)
        | DensityFunction::YClampedGradient { .. }
        | DensityFunction::BlendAlpha
        | DensityFunction::BlendOffset
        | DensityFunction::Beardifier
        | DensityFunction::EndIslands { .. } => eval_density_single_misc_terminal(df, chunk, pos),
        _ => unreachable!("non-terminal density function routed to single terminal evaluator"),
    }
}

fn eval_density_single_noise_terminal(
    df: DensityFunction,
    chunk: &NoiseChunk,
    pos: BlockPos,
) -> f64 {
    match df {
        DensityFunction::Noise {
            noise,
            xz_scale,
            y_scale,
        } => chunk.normal_noise_sample(
            noise,
            f64::from(pos.x) * xz_scale,
            f64::from(pos.y) * y_scale,
            f64::from(pos.z) * xz_scale,
        ),
        DensityFunction::ShiftA { noise } => {
            chunk.normal_noise_sample(noise, f64::from(pos.x) * 0.25, 0.0, f64::from(pos.z) * 0.25)
                * 4.0
        }
        DensityFunction::ShiftB { noise } => {
            chunk.normal_noise_sample(noise, f64::from(pos.z) * 0.25, f64::from(pos.x) * 0.25, 0.0)
                * 4.0
        }
        DensityFunction::Shift { noise } => {
            chunk.normal_noise_sample(
                noise,
                f64::from(pos.x) * 0.25,
                f64::from(pos.y) * 0.25,
                f64::from(pos.z) * 0.25,
            ) * 4.0
        }
        DensityFunction::ShiftedNoise {
            shift_x,
            shift_y,
            shift_z,
            xz_scale,
            y_scale,
            noise,
        } => eval_shifted_noise_single(
            ShiftedNoiseEval {
                shift_x,
                shift_y,
                shift_z,
                xz_scale,
                y_scale,
                noise,
            },
            chunk,
            pos,
        ),
        DensityFunction::BlendedNoise {
            xz_scale,
            y_scale,
            xz_factor,
            y_factor,
            smear_scale_multiplier,
        } => chunk.blended_noise_sample(
            xz_scale,
            y_scale,
            xz_factor,
            y_factor,
            smear_scale_multiplier,
            f64::from(pos.x),
            f64::from(pos.y),
            f64::from(pos.z),
        ),
        DensityFunction::WeirdScaledSampler {
            input,
            noise,
            rarity_mapper,
        } => {
            let input_value = eval_density_fn_single_point(*input, chunk, pos.x, pos.y, pos.z);
            weird_scaled_sampler_value(
                chunk,
                noise,
                rarity_mapper,
                pos.x,
                pos.y,
                pos.z,
                input_value,
            )
        }
        _ => unreachable!("non-noise density function routed to single noise terminal evaluator"),
    }
}

fn eval_density_single_misc_terminal(
    df: DensityFunction,
    chunk: &NoiseChunk,
    pos: BlockPos,
) -> f64 {
    match df {
        DensityFunction::FindTopSurface {
            density,
            upper_bound,
            lower_bound,
            cell_height,
        } => find_top_surface_compute(
            |sample_y| eval_density_fn_single_point(*density, chunk, pos.x, sample_y, pos.z),
            eval_density_fn_single_point(*upper_bound, chunk, pos.x, pos.y, pos.z),
            lower_bound,
            cell_height,
        ),
        DensityFunction::Constant(value) => value,
        DensityFunction::YClampedGradient { .. } => df.compute(pos.y),
        DensityFunction::BlendAlpha => 1.0,
        DensityFunction::BlendOffset | DensityFunction::Beardifier => 0.0,
        DensityFunction::EndIslands {
            seed: function_seed,
        } => end_island_density_sample(
            if function_seed == 0 {
                chunk.seed
            } else {
                function_seed
            },
            pos.x,
            pos.z,
        ),
        _ => unreachable!("non-misc density function routed to single misc terminal evaluator"),
    }
}

fn eval_shifted_noise_single(shifted: ShiftedNoiseEval, chunk: &NoiseChunk, pos: BlockPos) -> f64 {
    let sample_x = f64::from(pos.x) * shifted.xz_scale
        + eval_density_fn_single_point(*shifted.shift_x, chunk, pos.x, pos.y, pos.z);
    let sample_y = f64::from(pos.y) * shifted.y_scale
        + eval_density_fn_single_point(*shifted.shift_y, chunk, pos.x, pos.y, pos.z);
    let sample_z = f64::from(pos.z) * shifted.xz_scale
        + eval_density_fn_single_point(*shifted.shift_z, chunk, pos.x, pos.y, pos.z);
    chunk.normal_noise_sample(shifted.noise, sample_x, sample_y, sample_z)
}

pub(super) fn fill_density_array_with_interp(
    df: DensityFunction,
    chunk: &mut NoiseChunk,
    output: &mut [f64],
    mode: DensityArrayFillMode,
) {
    match df {
        DensityFunction::Reference(id) => {
            if let Some(entry) = builtin_density_function(id) {
                fill_density_array_with_interp(entry.function, chunk, output, mode);
            } else {
                output.fill(0.0);
            }
        }
        DensityFunction::Marker { kind, input } => {
            fill_density_array_marker_with_interp(df, kind, input, chunk, output, mode);
        }
        DensityFunction::BlendDensity { .. }
        | DensityFunction::Clamp { .. }
        | DensityFunction::Mapped { .. }
        | DensityFunction::Binary { .. }
        | DensityFunction::RangeChoice { .. } => {
            fill_density_array_composed_with_interp(df, chunk, output, mode);
        }
        other => fill_density_array_terminal_with_interp(other, chunk, output, mode),
    }
}

fn fill_density_array_marker_with_interp(
    marker_df: DensityFunction,
    kind: DensityMarker,
    input: &'static DensityFunction,
    chunk: &mut NoiseChunk,
    output: &mut [f64],
    mode: DensityArrayFillMode,
) {
    match kind {
        DensityMarker::CacheOnce => fill_cache_once_density_array(input, chunk, output, mode),
        DensityMarker::Interpolated => {
            fill_interpolated_marker_array(marker_df, input, chunk, output, mode)
        }
        DensityMarker::CacheAllInCell => {
            fill_cache_all_in_cell_density_array(input, chunk, output, mode);
        }
        DensityMarker::Cache2D => fill_density_array_with_interp(*input, chunk, output, mode),
        DensityMarker::FlatCache => fill_density_array_direct(marker_df, chunk, output, mode),
    }
}

fn fill_cache_once_density_array(
    input: &'static DensityFunction,
    chunk: &mut NoiseChunk,
    output: &mut [f64],
    mode: DensityArrayFillMode,
) {
    let ptr = input as *const DensityFunction as usize;
    let Some(cache_index) = lookup_density_index(&chunk.cache_once_by_ptr, ptr) else {
        chunk
            .cache_once_array_misses
            .set(chunk.cache_once_array_misses.get().saturating_add(1));
        fill_density_array_with_interp(*input, chunk, output, mode);
        return;
    };
    if copy_cache_once_array_if_present(chunk, output, cache_index) {
        chunk
            .cache_once_array_hits
            .set(chunk.cache_once_array_hits.get().saturating_add(1));
        return;
    }

    chunk
        .cache_once_array_misses
        .set(chunk.cache_once_array_misses.get().saturating_add(1));
    fill_density_array_with_interp(*input, chunk, output, mode);
    store_cache_once_array(chunk, output, cache_index);
}

fn copy_cache_once_array_if_present(
    chunk: &NoiseChunk,
    output: &mut [f64],
    cache_index: usize,
) -> bool {
    let cache = chunk.cache_once_values.borrow();
    let state = &cache[cache_index];
    if let Some(array) = (state.last_array_counter == chunk.array_interpolation_counter)
        .then_some(state.last_array.as_deref())
        .flatten()
        .filter(|array| array.len() == output.len())
    {
        output.copy_from_slice(array);
        true
    } else {
        false
    }
}

fn store_cache_once_array(chunk: &NoiseChunk, output: &[f64], cache_index: usize) {
    let mut cache = chunk.cache_once_values.borrow_mut();
    let state = &mut cache[cache_index];
    state.last_array_counter = chunk.array_interpolation_counter;
    if let Some(array) = state
        .last_array
        .as_mut()
        .filter(|array| array.len() == output.len())
    {
        array.copy_from_slice(output);
    } else {
        state.last_array = Some(output.to_vec());
    }
}

fn fill_interpolated_marker_array(
    marker_df: DensityFunction,
    input: &'static DensityFunction,
    chunk: &mut NoiseChunk,
    output: &mut [f64],
    mode: DensityArrayFillMode,
) {
    match mode {
        // While building an interpolator's corner slices, evaluate the wrapped
        // function directly at the cell corners. Once a block-cell cache is being
        // populated, the interpolators have already selected their current cell, so
        // sampling the marker itself preserves Java's trilinear interpolation.
        DensityArrayFillMode::Slice { .. } => {
            fill_density_array_with_interp(*input, chunk, output, mode);
        }
        DensityArrayFillMode::Cell => {
            fill_density_array_direct(marker_df, chunk, output, mode);
        }
    }
}

fn fill_cache_all_in_cell_density_array(
    input: &'static DensityFunction,
    chunk: &mut NoiseChunk,
    output: &mut [f64],
    mode: DensityArrayFillMode,
) {
    if !chunk.filling_cell_cache.get() {
        let ptr = input as *const DensityFunction as usize;
        if let Some(cache_index) = lookup_density_index(&chunk.cache_all_by_ptr, ptr) {
            if chunk.cache_all_in_cell[cache_index].values.len() == output.len() {
                output.copy_from_slice(&chunk.cache_all_in_cell[cache_index].values);
                return;
            }
        }
    }
    fill_density_array_with_interp(*input, chunk, output, mode);
}

fn fill_density_array_composed_with_interp(
    df: DensityFunction,
    chunk: &mut NoiseChunk,
    output: &mut [f64],
    mode: DensityArrayFillMode,
) {
    match df {
        DensityFunction::BlendDensity { input } => {
            fill_density_array_with_interp(*input, chunk, output, mode);
        }
        DensityFunction::Clamp { input, min, max } => {
            fill_density_array_with_interp(*input, chunk, output, mode);
            for value in output {
                *value = value.clamp(min, max);
            }
        }
        DensityFunction::Mapped { kind, input } => {
            fill_density_array_with_interp(*input, chunk, output, mode);
            for value in output {
                *value = kind.transform(*value);
            }
        }
        DensityFunction::Binary {
            kind,
            argument1,
            argument2,
        } => fill_binary_density_array(kind, argument1, argument2, chunk, output, mode),
        DensityFunction::RangeChoice {
            input,
            min_inclusive,
            max_exclusive,
            when_in_range,
            when_out_of_range,
        } => fill_range_choice_density_array(
            DensityArrayRangeChoice {
                input,
                min_inclusive,
                max_exclusive,
                when_in_range,
                when_out_of_range,
            },
            chunk,
            output,
            mode,
        ),
        _ => unreachable!("non-composed density function routed to array composed evaluator"),
    }
}

fn fill_binary_density_array(
    kind: BinaryDensityFunction,
    argument1: &'static DensityFunction,
    argument2: &'static DensityFunction,
    chunk: &mut NoiseChunk,
    output: &mut [f64],
    mode: DensityArrayFillMode,
) {
    fill_density_array_with_interp(*argument1, chunk, output, mode);
    match kind {
        BinaryDensityFunction::Add => add_density_array_argument(argument2, chunk, output, mode),
        BinaryDensityFunction::Mul => apply_density_array_mul(argument2, chunk, output, mode),
        BinaryDensityFunction::Min => apply_density_array_min(argument2, chunk, output, mode),
        BinaryDensityFunction::Max => apply_density_array_max(argument2, chunk, output, mode),
    }
}

fn add_density_array_argument(
    argument: &'static DensityFunction,
    chunk: &mut NoiseChunk,
    output: &mut [f64],
    mode: DensityArrayFillMode,
) {
    let mut second = chunk.take_density_array_scratch(output.len());
    fill_density_array_with_interp(*argument, chunk, &mut second, mode);
    for (value, second) in output.iter_mut().zip(second.iter()) {
        *value += *second;
    }
    chunk.return_density_array_scratch(second);
}

fn apply_density_array_mul(
    argument: &'static DensityFunction,
    chunk: &mut NoiseChunk,
    output: &mut [f64],
    mode: DensityArrayFillMode,
) {
    for (index, value) in output.iter_mut().enumerate() {
        if *value == 0.0 {
            continue;
        }
        let (pos_x, pos_y, pos_z) = prepare_density_array_context(chunk, mode, index);
        *value *= eval_density_fn_with_interp(*argument, chunk, pos_x, pos_y, pos_z);
    }
}

fn apply_density_array_min(
    argument: &'static DensityFunction,
    chunk: &mut NoiseChunk,
    output: &mut [f64],
    mode: DensityArrayFillMode,
) {
    let second_min = chunk.density_value_bounds(argument).0;
    for (index, value) in output.iter_mut().enumerate() {
        if *value < second_min {
            continue;
        }
        let (pos_x, pos_y, pos_z) = prepare_density_array_context(chunk, mode, index);
        *value = value.min(eval_density_fn_with_interp(
            *argument, chunk, pos_x, pos_y, pos_z,
        ));
    }
}

fn apply_density_array_max(
    argument: &'static DensityFunction,
    chunk: &mut NoiseChunk,
    output: &mut [f64],
    mode: DensityArrayFillMode,
) {
    let second_max = chunk.density_value_bounds(argument).1;
    for (index, value) in output.iter_mut().enumerate() {
        if *value > second_max {
            continue;
        }
        let (pos_x, pos_y, pos_z) = prepare_density_array_context(chunk, mode, index);
        *value = value.max(eval_density_fn_with_interp(
            *argument, chunk, pos_x, pos_y, pos_z,
        ));
    }
}

#[derive(Clone, Copy)]
struct DensityArrayRangeChoice {
    input: &'static DensityFunction,
    min_inclusive: f64,
    max_exclusive: f64,
    when_in_range: &'static DensityFunction,
    when_out_of_range: &'static DensityFunction,
}

fn fill_range_choice_density_array(
    choice: DensityArrayRangeChoice,
    chunk: &mut NoiseChunk,
    output: &mut [f64],
    mode: DensityArrayFillMode,
) {
    let mut selector = chunk.take_density_array_scratch(output.len());
    fill_density_array_with_interp(*choice.input, chunk, &mut selector, mode);
    for index in 0..output.len() {
        let (pos_x, pos_y, pos_z) = prepare_density_array_context(chunk, mode, index);
        output[index] =
            if selector[index] >= choice.min_inclusive && selector[index] < choice.max_exclusive {
                eval_density_fn_with_interp(*choice.when_in_range, chunk, pos_x, pos_y, pos_z)
            } else {
                eval_density_fn_with_interp(*choice.when_out_of_range, chunk, pos_x, pos_y, pos_z)
            };
    }
    chunk.return_density_array_scratch(selector);
}

fn fill_density_array_terminal_with_interp(
    df: DensityFunction,
    chunk: &mut NoiseChunk,
    output: &mut [f64],
    mode: DensityArrayFillMode,
) {
    match df {
        DensityFunction::WeirdScaledSampler {
            input,
            noise,
            rarity_mapper,
        } => fill_weird_scaled_sampler_array(input, noise, rarity_mapper, chunk, output, mode),
        DensityFunction::Constant(value) => output.fill(value),
        DensityFunction::BlendAlpha => output.fill(1.0),
        DensityFunction::BlendOffset | DensityFunction::Beardifier => output.fill(0.0),
        _ => fill_density_array_direct(df, chunk, output, mode),
    }
}

fn fill_weird_scaled_sampler_array(
    input: &'static DensityFunction,
    noise: &'static str,
    rarity_mapper: RarityValueMapper,
    chunk: &mut NoiseChunk,
    output: &mut [f64],
    mode: DensityArrayFillMode,
) {
    fill_density_array_with_interp(*input, chunk, output, mode);
    for (index, output_value) in output.iter_mut().enumerate() {
        let input_value = *output_value;
        let (pos_x, pos_y, pos_z) = prepare_density_array_context(chunk, mode, index);
        *output_value = weird_scaled_sampler_value(
            chunk,
            noise,
            rarity_mapper,
            pos_x,
            pos_y,
            pos_z,
            input_value,
        );
    }
}

pub(super) fn prepare_density_array_context(
    chunk: &mut NoiseChunk,
    mode: DensityArrayFillMode,
    index: usize,
) -> (i32, i32, i32) {
    match mode {
        DensityArrayFillMode::Slice { block_x, block_z } => {
            let block_y = (chunk.cell_noise_min_y + index as i32) * chunk.cell_height;
            chunk.cell_start_block_x = block_x;
            chunk.cell_start_block_y = block_y;
            chunk.cell_start_block_z = block_z;
            chunk.interpolation_counter += 1;
            chunk.in_cell_x = 0;
            chunk.in_cell_y = 0;
            chunk.in_cell_z = 0;
            chunk.array_index = index;
            (block_x, block_y, block_z)
        }
        DensityArrayFillMode::Cell => {
            let cell_width = chunk.cell_width as usize;
            let y_stride = cell_width * cell_width;
            let y_from_top = index / y_stride;
            let xz_index = index % y_stride;
            let x_in_cell = xz_index / cell_width;
            let z_in_cell = xz_index % cell_width;
            let y_in_cell = chunk.cell_height - 1 - y_from_top as i32;
            chunk.in_cell_x = x_in_cell as i32;
            chunk.in_cell_y = y_in_cell;
            chunk.in_cell_z = z_in_cell as i32;
            chunk.array_index = index;
            (
                chunk.cell_start_block_x + chunk.in_cell_x,
                chunk.cell_start_block_y + chunk.in_cell_y,
                chunk.cell_start_block_z + chunk.in_cell_z,
            )
        }
    }
}

pub(super) fn fill_density_array_direct(
    df: DensityFunction,
    chunk: &mut NoiseChunk,
    output: &mut [f64],
    mode: DensityArrayFillMode,
) {
    match mode {
        DensityArrayFillMode::Slice { block_x, block_z } => {
            for (cell_y_index, value) in output.iter_mut().enumerate() {
                let block_y = (chunk.cell_noise_min_y + cell_y_index as i32) * chunk.cell_height;
                chunk.cell_start_block_x = block_x;
                chunk.cell_start_block_y = block_y;
                chunk.cell_start_block_z = block_z;
                chunk.interpolation_counter += 1;
                chunk.in_cell_x = 0;
                chunk.in_cell_y = 0;
                chunk.in_cell_z = 0;
                chunk.array_index = cell_y_index;
                *value = eval_density_fn_with_interp(df, chunk, block_x, block_y, block_z);
            }
        }
        DensityArrayFillMode::Cell => {
            chunk.array_index = 0;
            for y_in_cell in (0..chunk.cell_height).rev() {
                chunk.in_cell_y = y_in_cell;
                let pos_y = chunk.cell_start_block_y + y_in_cell;
                for x_in_cell in 0..chunk.cell_width {
                    chunk.in_cell_x = x_in_cell;
                    let pos_x = chunk.cell_start_block_x + x_in_cell;
                    for z_in_cell in 0..chunk.cell_width {
                        chunk.in_cell_z = z_in_cell;
                        let pos_z = chunk.cell_start_block_z + z_in_cell;
                        output[chunk.array_index] =
                            eval_density_fn_with_interp(df, chunk, pos_x, pos_y, pos_z);
                        chunk.array_index += 1;
                    }
                }
            }
        }
    }
}

// ── NoiseInterpolatorState ────────────────────────────────────────────────────
