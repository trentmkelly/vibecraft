use super::*;

/// Pack a `(block_x, block_z)` pair into a single `i64` key.
///
/// Matches Java `ColumnPos.asLong(blockX, blockZ)`:
/// `(blockX & 0xFFFF_FFFF_L) | ((long) blockZ << 32)`.
pub(super) fn pack_column(x: i32, z: i32) -> i64 {
    ((x as i64) & 0xFFFF_FFFF) | ((z as i64) << 32)
}

fn density_cache_key(df: &DensityFunction, kind: DensityMarker) -> usize {
    (df as *const DensityFunction as usize) ^ ((kind as usize) << 3)
}

pub(super) fn density_function_key(df: &DensityFunction) -> usize {
    df as *const DensityFunction as usize
}

pub(super) fn lookup_density_index(entries: &HashMap<usize, usize>, key: usize) -> Option<usize> {
    entries.get(&key).copied()
}

fn build_density_lookup(inputs: &[&'static DensityFunction]) -> HashMap<usize, usize> {
    inputs
        .iter()
        .enumerate()
        .map(|(i, &fn_ref)| (fn_ref as *const DensityFunction as usize, i))
        .collect()
}

fn collect_marker_inputs(
    df: DensityFunction,
    target: DensityMarker,
    out: &mut Vec<&'static DensityFunction>,
) {
    match df {
        DensityFunction::Marker { kind, input } => {
            if kind == target {
                let ptr = input as *const DensityFunction as usize;
                if !out
                    .iter()
                    .any(|existing| *existing as *const DensityFunction as usize == ptr)
                {
                    out.push(input);
                }
            }
            collect_marker_inputs(*input, target, out);
        }
        DensityFunction::BlendDensity { input }
        | DensityFunction::Clamp { input, .. }
        | DensityFunction::Mapped { input, .. }
        | DensityFunction::WeirdScaledSampler { input, .. } => {
            collect_marker_inputs(*input, target, out);
        }
        DensityFunction::Binary {
            argument1,
            argument2,
            ..
        } => {
            collect_marker_inputs(*argument1, target, out);
            collect_marker_inputs(*argument2, target, out);
        }
        DensityFunction::RangeChoice {
            input,
            when_in_range,
            when_out_of_range,
            ..
        } => {
            collect_marker_inputs(*input, target, out);
            collect_marker_inputs(*when_in_range, target, out);
            collect_marker_inputs(*when_out_of_range, target, out);
        }
        DensityFunction::ShiftedNoise {
            shift_x,
            shift_y,
            shift_z,
            ..
        } => {
            collect_marker_inputs(*shift_x, target, out);
            collect_marker_inputs(*shift_y, target, out);
            collect_marker_inputs(*shift_z, target, out);
        }
        DensityFunction::FindTopSurface {
            density,
            upper_bound,
            ..
        } => {
            collect_marker_inputs(*density, target, out);
            collect_marker_inputs(*upper_bound, target, out);
        }
        DensityFunction::Reference(id) => {
            if let Some(entry) = builtin_density_function(id) {
                collect_marker_inputs(entry.function, target, out);
            }
        }
        _ => {}
    }
}

/// Walk a `DensityFunction` tree depth-first and collect the `&'static`
/// inner functions of every `Marker(Interpolated)` node.
///
/// Duplicates (by pointer identity) are not added twice — this matches Java's
/// `HashMap<DensityFunction, DensityFunction>` deduplication in `NoiseChunk.wrap`.
fn collect_interpolated_inputs(df: DensityFunction, out: &mut Vec<&'static DensityFunction>) {
    collect_marker_inputs(df, DensityMarker::Interpolated, out);
}

#[derive(Clone)]
pub(super) struct NoiseChunkTemplate {
    pub(super) interpolated_inputs: Arc<[&'static DensityFunction]>,
    pub(super) cache_all_inputs: Arc<[&'static DensityFunction]>,
    pub(super) flat_cache_inputs: Arc<[&'static DensityFunction]>,
    pub(super) cache_2d_inputs: Arc<[&'static DensityFunction]>,
    pub(super) cache_once_inputs: Arc<[&'static DensityFunction]>,
    pub(super) interp_by_ptr: Arc<HashMap<usize, usize>>,
    pub(super) cache_all_by_ptr: Arc<HashMap<usize, usize>>,
    pub(super) flat_cache_by_ptr: Arc<HashMap<usize, usize>>,
    pub(super) cache_2d_by_ptr: Arc<HashMap<usize, usize>>,
    pub(super) cache_once_by_ptr: Arc<HashMap<usize, usize>>,
}

static NOISE_CHUNK_TEMPLATE_CACHE: OnceLock<Mutex<HashMap<&'static str, NoiseChunkTemplate>>> =
    OnceLock::new();

pub(super) fn noise_chunk_template(
    settings_id: &'static str,
    noise_router: NoiseRouter,
) -> NoiseChunkTemplate {
    let cache = NOISE_CHUNK_TEMPLATE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(cache) = cache.lock() {
        if let Some(template) = cache.get(settings_id) {
            return template.clone();
        }
    }

    let mut interpolated_inputs: Vec<&'static DensityFunction> = Vec::new();
    for function in [
        noise_router.final_density,
        noise_router.vein_toggle,
        noise_router.vein_ridged,
        noise_router.vein_gap,
        noise_router.barrier,
        noise_router.fluid_level_floodedness,
        noise_router.fluid_level_spread,
        noise_router.lava,
        noise_router.erosion,
        noise_router.depth,
        noise_router.preliminary_surface_level,
    ] {
        collect_interpolated_inputs(function, &mut interpolated_inputs);
    }

    let mut cache_all_inputs: Vec<&'static DensityFunction> = Vec::new();
    let mut flat_cache_inputs: Vec<&'static DensityFunction> = Vec::new();
    let mut cache_2d_inputs: Vec<&'static DensityFunction> = Vec::new();
    let mut cache_once_inputs: Vec<&'static DensityFunction> = Vec::new();
    for function in [
        noise_router.final_density,
        noise_router.vein_toggle,
        noise_router.vein_ridged,
        noise_router.vein_gap,
        noise_router.barrier,
        noise_router.fluid_level_floodedness,
        noise_router.fluid_level_spread,
        noise_router.lava,
        noise_router.erosion,
        noise_router.depth,
        noise_router.preliminary_surface_level,
    ] {
        collect_marker_inputs(
            function,
            DensityMarker::CacheAllInCell,
            &mut cache_all_inputs,
        );
        collect_marker_inputs(function, DensityMarker::FlatCache, &mut flat_cache_inputs);
        collect_marker_inputs(function, DensityMarker::Cache2D, &mut cache_2d_inputs);
        collect_marker_inputs(function, DensityMarker::CacheOnce, &mut cache_once_inputs);
    }

    let template = NoiseChunkTemplate {
        interp_by_ptr: Arc::new(build_density_lookup(&interpolated_inputs)),
        cache_all_by_ptr: Arc::new(build_density_lookup(&cache_all_inputs)),
        flat_cache_by_ptr: Arc::new(build_density_lookup(&flat_cache_inputs)),
        cache_2d_by_ptr: Arc::new(build_density_lookup(&cache_2d_inputs)),
        cache_once_by_ptr: Arc::new(build_density_lookup(&cache_once_inputs)),
        interpolated_inputs: Arc::from(interpolated_inputs.into_boxed_slice()),
        cache_all_inputs: Arc::from(cache_all_inputs.into_boxed_slice()),
        flat_cache_inputs: Arc::from(flat_cache_inputs.into_boxed_slice()),
        cache_2d_inputs: Arc::from(cache_2d_inputs.into_boxed_slice()),
        cache_once_inputs: Arc::from(cache_once_inputs.into_boxed_slice()),
    };
    if let Ok(mut cache) = cache.lock() {
        cache.insert(settings_id, template.clone());
    }
    template
}
