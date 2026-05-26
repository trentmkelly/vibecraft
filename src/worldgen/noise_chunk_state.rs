use super::*;

/// Per-density-function trilinear interpolator.
///
/// Mirrors Java's `NoiseChunk.NoiseInterpolator`.
///
/// Two "slice" arrays (`slice0` / `slice1`) hold sampled corner values along
/// the current and next X-cell-column respectively.  Each slice is indexed as
/// `slice[z_corner][y_corner]` where both indices run from `0` to `cell_count`
/// (inclusive).
///
/// During block iteration the 8 cell-corner values are loaded by
/// `select_cell_yz`, then the Y/X/Z lerp steps update partial accumulators
/// until `value` holds the fully-interpolated density.
pub(super) struct NoiseInterpolatorState {
    /// The inner density function that is sampled at cell corners.
    inner_fn: &'static DensityFunction,
    /// `slice0[z_idx][y_idx]`: corner values for the *current* X cell column.
    slice0: Vec<Vec<f64>>,
    /// `slice1[z_idx][y_idx]`: corner values for the *next* X cell column.
    slice1: Vec<Vec<f64>>,
    // Eight cell-corner values (populated by `select_cell_yz`).
    // Naming: noise_XYZ where X/Y/Z = 0 (current cell edge) or 1 (next edge).
    noise000: f64,
    noise001: f64,
    noise100: f64,
    noise101: f64,
    noise010: f64,
    noise011: f64,
    noise110: f64,
    noise111: f64,
    // Partial lerp accumulators — written by update_for_y / update_for_x.
    value_xz00: f64,
    value_xz10: f64,
    value_xz01: f64,
    value_xz11: f64,
    value_z0: f64,
    value_z1: f64,
    /// Fully-interpolated value for the current block position (written by
    /// `update_for_z`).
    value: f64,
}

impl NoiseInterpolatorState {
    fn new(cell_count_y: usize, cell_count_xz: usize, inner_fn: &'static DensityFunction) -> Self {
        let sz = cell_count_xz + 1; // z corners
        let sy = cell_count_y + 1; // y corners
        Self {
            inner_fn,
            slice0: (0..sz).map(|_| vec![0.0; sy]).collect(),
            slice1: (0..sz).map(|_| vec![0.0; sy]).collect(),
            noise000: 0.0,
            noise001: 0.0,
            noise100: 0.0,
            noise101: 0.0,
            noise010: 0.0,
            noise011: 0.0,
            noise110: 0.0,
            noise111: 0.0,
            value_xz00: 0.0,
            value_xz10: 0.0,
            value_xz01: 0.0,
            value_xz11: 0.0,
            value_z0: 0.0,
            value_z1: 0.0,
            value: 0.0,
        }
    }

    /// Load the 8 corner values for the cell at `(cell_y_idx, cell_z_idx)`.
    ///
    /// Mirrors Java `NoiseInterpolator.selectCellYZ`.
    fn select_cell_yz(&mut self, cell_y_idx: usize, cell_z_idx: usize) {
        self.noise000 = self.slice0[cell_z_idx][cell_y_idx];
        self.noise001 = self.slice0[cell_z_idx + 1][cell_y_idx];
        self.noise100 = self.slice1[cell_z_idx][cell_y_idx];
        self.noise101 = self.slice1[cell_z_idx + 1][cell_y_idx];
        self.noise010 = self.slice0[cell_z_idx][cell_y_idx + 1];
        self.noise011 = self.slice0[cell_z_idx + 1][cell_y_idx + 1];
        self.noise110 = self.slice1[cell_z_idx][cell_y_idx + 1];
        self.noise111 = self.slice1[cell_z_idx + 1][cell_y_idx + 1];
    }

    /// Lerp along Y: populate `value_xz*` from the 8 corner values.
    ///
    /// `factor_y` is `y_in_cell / cell_height` (0.0 at bottom of cell → 1.0 at top).
    ///
    /// Mirrors Java `NoiseInterpolator.updateForY`.
    fn update_for_y(&mut self, factor_y: f64) {
        self.value_xz00 = lerp(factor_y, self.noise000, self.noise010);
        self.value_xz10 = lerp(factor_y, self.noise100, self.noise110);
        self.value_xz01 = lerp(factor_y, self.noise001, self.noise011);
        self.value_xz11 = lerp(factor_y, self.noise101, self.noise111);
    }

    /// Lerp along X: populate `value_z*` from `value_xz*`.
    ///
    /// `factor_x` is `x_in_cell / cell_width`.
    ///
    /// Mirrors Java `NoiseInterpolator.updateForX`.
    fn update_for_x(&mut self, factor_x: f64) {
        self.value_z0 = lerp(factor_x, self.value_xz00, self.value_xz10);
        self.value_z1 = lerp(factor_x, self.value_xz01, self.value_xz11);
    }

    /// Lerp along Z: write the fully-interpolated `value`.
    ///
    /// `factor_z` is `z_in_cell / cell_width`.
    ///
    /// Mirrors Java `NoiseInterpolator.updateForZ`.
    fn update_for_z(&mut self, factor_z: f64) {
        self.value = lerp(factor_z, self.value_z0, self.value_z1);
    }

    /// Swap `slice0` and `slice1` so that the previously-filled "next" slice
    /// becomes the "current" slice at the start of the next X-cell-column.
    ///
    /// Mirrors Java `NoiseInterpolator.swapSlices`.
    fn swap_slices(&mut self) {
        std::mem::swap(&mut self.slice0, &mut self.slice1);
    }
}

pub(super) struct CacheAllInCellState {
    pub(super) inner_fn: &'static DensityFunction,
    pub(super) values: Vec<f64>,
}

impl CacheAllInCellState {
    fn new(cell_width: i32, cell_height: i32, inner_fn: &'static DensityFunction) -> Self {
        let len = (cell_width * cell_width * cell_height) as usize;
        Self {
            inner_fn,
            values: vec![0.0; len],
        }
    }
}

pub(super) struct Cache2DState {
    pub(super) inner_fn: &'static DensityFunction,
    pub(super) last_pos_2d: i64,
    pub(super) last_value: f64,
}

impl Cache2DState {
    fn new(inner_fn: &'static DensityFunction) -> Self {
        Self {
            inner_fn,
            last_pos_2d: i64::MIN,
            last_value: 0.0,
        }
    }
}

pub(super) struct FlatCacheState {
    pub(super) inner_fn: &'static DensityFunction,
    pub(super) values: Vec<f64>,
    pub(super) size_xz: usize,
}

impl FlatCacheState {
    fn new(
        chunk_min_block_x: i32,
        chunk_min_block_z: i32,
        noise_size_xz: i32,
        seed: i64,
        settings: NoiseGeneratorSettings,
        inner_fn: &'static DensityFunction,
    ) -> Self {
        let size_xz = (noise_size_xz + 1) as usize;
        let first_noise_x = chunk_min_block_x >> 2;
        let first_noise_z = chunk_min_block_z >> 2;
        let mut values = vec![0.0; size_xz * size_xz];
        for x in 0..=noise_size_xz {
            let block_x = (first_noise_x + x) << 2;
            for z in 0..=noise_size_xz {
                let block_z = (first_noise_z + z) << 2;
                values[x as usize + z as usize * size_xz] =
                    inner_fn.compute_with_noise(seed, settings, block_x, 0, block_z);
            }
        }
        Self {
            inner_fn,
            values,
            size_xz,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct NoiseChunkFillStats {
    pub(super) full_noise_cache_ms: u128,
    pub(super) full_noise_cache_us: u128,
    pub(super) full_noise_cache_fills: usize,
    pub(super) vein_noise_cache_ms: u128,
    pub(super) vein_noise_cache_us: u128,
    pub(super) vein_noise_cache_fills: usize,
    pub(super) cache_once_scalar_hits: usize,
    pub(super) cache_once_scalar_misses: usize,
    pub(super) cache_once_array_hits: usize,
    pub(super) cache_once_array_misses: usize,
}

// ── NoiseChunk ────────────────────────────────────────────────────────────────

/// Cell-based sampling context for one chunk's noise generation.
///
/// Mirrors Java's `NoiseChunk` (inner interpolation state only; Aquifer and
/// Beardifier are not yet wired).
///
/// The overworld uses 4-wide × 8-tall cells (`cell_width = 4`, `cell_height = 8`).
/// For each X-cell-column, `advance_cell_x` fills the *next* slice, then
/// `swap_slices` promotes it to the *current* slice.  Within each cell, values
/// are trilinearly interpolated by successive calls to `update_for_y`,
/// `update_for_x`, and `update_for_z`.
pub struct NoiseChunk {
    pub cell_width: i32,
    pub cell_height: i32,
    /// Number of cells along X and Z (= 16 / cell_width).
    pub cell_count_xz: i32,
    /// Number of cells along Y (= height / cell_height).
    pub cell_count_y: i32,
    /// Lowest Y cell index (= min_y / cell_height, using floor division).
    pub cell_noise_min_y: i32,
    /// Cell X index of the chunk's west edge.
    pub first_cell_x: i32,
    /// Cell Z index of the chunk's north edge.
    pub first_cell_z: i32,
    /// Quart-block count along XZ within the chunk (= cell_count_xz * cell_width >> 2).
    pub noise_size_xz: i32,

    /// One interpolator per `Interpolated`-marked inner function found in
    /// `final_density`.
    pub(super) interpolators: Vec<NoiseInterpolatorState>,
    /// Maps inner-function pointer address → `interpolators` index so that
    /// `eval_density_fn_with_interp` can look up the current interpolated value
    /// in O(1).
    pub(super) interp_by_ptr: Arc<HashMap<usize, usize>>,
    pub(super) cache_all_in_cell: Vec<CacheAllInCellState>,
    pub(super) cache_all_by_ptr: Arc<HashMap<usize, usize>>,
    pub(super) cache_2d: RefCell<Vec<Cache2DState>>,
    pub(super) cache_2d_by_ptr: Arc<HashMap<usize, usize>>,
    pub(super) flat_cache: Vec<FlatCacheState>,
    pub(super) flat_cache_by_ptr: Arc<HashMap<usize, usize>>,
    pub(super) full_noise_values: Vec<f64>,
    pub(super) vein_toggle_values: Vec<f64>,
    pub(super) vein_ridged_values: Vec<f64>,
    pub(super) vein_gap_values: Vec<f64>,
    pub(super) vein_toggle_interp_index: Option<usize>,
    pub(super) vein_a_interp_index: Option<usize>,
    pub(super) vein_b_interp_index: Option<usize>,

    // Current position tracking (updated during the iteration loops).
    pub cell_start_block_x: i32,
    pub cell_start_block_y: i32,
    pub cell_start_block_z: i32,
    pub in_cell_x: i32,
    pub in_cell_y: i32,
    pub in_cell_z: i32,

    /// `true` while inside the X-cell-column iteration loop.
    pub(super) interpolating: bool,

    /// Cache of preliminary surface levels, keyed by `pack_column(blockX, blockZ)`.
    pub(super) prelim_surface_cache: RefCell<HashMap<i64, i32>>,
    pub(super) cache_once_values: RefCell<Vec<CacheOnceState>>,
    pub(super) cache_once_by_ptr: Arc<HashMap<usize, usize>>,
    pub(super) normal_noise_cache: RefCell<HashMap<&'static str, NormalNoiseSnapshot>>,
    pub(super) blended_noise_cache: RefCell<Vec<([u64; 5], BlendedNoiseSnapshot)>>,
    pub(super) density_value_bounds_cache: RefCell<HashMap<usize, (f64, f64)>>,
    pub(super) terrain_spline_cache: RefCell<HashMap<TerrainSplineKind, TerrainCubicSpline>>,
    pub(super) density_array_scratch: Vec<Vec<f64>>,
    pub(super) filling_cell_cache: Cell<bool>,
    pub(super) filling_cell: bool,
    pub(super) interpolation_counter: i64,
    pub(super) array_interpolation_counter: i64,
    pub(super) array_index: usize,
    pub(super) fill_stats: NoiseChunkFillStats,
    pub(super) cache_once_scalar_hits: Cell<usize>,
    pub(super) cache_once_scalar_misses: Cell<usize>,
    pub(super) cache_once_array_hits: Cell<usize>,
    pub(super) cache_once_array_misses: Cell<usize>,

    pub seed: i64,
    pub settings: NoiseGeneratorSettings,
    pub noise_router: NoiseRouter,
}

struct NoiseChunkGrid {
    cell_width: i32,
    cell_height: i32,
    cell_count_xz: i32,
    cell_count_y: i32,
    cell_noise_min_y: i32,
    first_cell_x: i32,
    first_cell_z: i32,
    noise_size_xz: i32,
}

struct NoiseChunkAllocatedState {
    interpolators: Vec<NoiseInterpolatorState>,
    cache_all_in_cell: Vec<CacheAllInCellState>,
    cache_2d: Vec<Cache2DState>,
    cache_once_values: Vec<CacheOnceState>,
    flat_cache: Vec<FlatCacheState>,
    full_noise_values: Vec<f64>,
    vein_toggle_values: Vec<f64>,
    vein_ridged_values: Vec<f64>,
    vein_gap_values: Vec<f64>,
}

struct NoiseChunkVeinInterpolatorIndexes {
    toggle: Option<usize>,
    vein_a: Option<usize>,
    vein_b: Option<usize>,
}

fn noise_chunk_grid(
    chunk_min_block_x: i32,
    chunk_min_block_z: i32,
    settings: NoiseGeneratorSettings,
) -> NoiseChunkGrid {
    let cell_width = settings.noise.cell_width();
    let cell_height = settings.noise.cell_height();
    let cell_count_xz = 16 / cell_width;
    let cell_count_y = settings.noise.height / cell_height;
    let cell_noise_min_y = settings.noise.min_y.div_euclid(cell_height);
    let first_cell_x = chunk_min_block_x.div_euclid(cell_width);
    let first_cell_z = chunk_min_block_z.div_euclid(cell_width);
    let noise_size_xz = (cell_count_xz * cell_width) >> 2;

    NoiseChunkGrid {
        cell_width,
        cell_height,
        cell_count_xz,
        cell_count_y,
        cell_noise_min_y,
        first_cell_x,
        first_cell_z,
        noise_size_xz,
    }
}

fn allocate_noise_chunk_state(
    chunk_min_block_x: i32,
    chunk_min_block_z: i32,
    settings: NoiseGeneratorSettings,
    seed: i64,
    grid: &NoiseChunkGrid,
    template: &NoiseChunkTemplate,
) -> NoiseChunkAllocatedState {
    // Allocate one interpolator per unique inner function.
    let interpolators: Vec<NoiseInterpolatorState> = template
        .interpolated_inputs
        .iter()
        .map(|&fn_ref| {
            NoiseInterpolatorState::new(
                grid.cell_count_y as usize,
                grid.cell_count_xz as usize,
                fn_ref,
            )
        })
        .collect();
    let cache_all_in_cell: Vec<CacheAllInCellState> = template
        .cache_all_inputs
        .iter()
        .map(|&fn_ref| CacheAllInCellState::new(grid.cell_width, grid.cell_height, fn_ref))
        .collect();
    let cache_2d: Vec<Cache2DState> = template
        .cache_2d_inputs
        .iter()
        .map(|&fn_ref| Cache2DState::new(fn_ref))
        .collect();
    let cache_once_values: Vec<CacheOnceState> = template
        .cache_once_inputs
        .iter()
        .map(|_| CacheOnceState::default())
        .collect();
    let flat_cache: Vec<FlatCacheState> = template
        .flat_cache_inputs
        .iter()
        .map(|&fn_ref| {
            FlatCacheState::new(
                chunk_min_block_x,
                chunk_min_block_z,
                grid.noise_size_xz,
                seed,
                settings,
                fn_ref,
            )
        })
        .collect();
    let cell_value_count = (grid.cell_width * grid.cell_width * grid.cell_height) as usize;

    NoiseChunkAllocatedState {
        interpolators,
        cache_all_in_cell,
        cache_2d,
        cache_once_values,
        flat_cache,
        full_noise_values: vec![0.0; cell_value_count],
        vein_toggle_values: vec![0.0; cell_value_count],
        vein_ridged_values: vec![0.0; cell_value_count],
        vein_gap_values: vec![0.0; cell_value_count],
    }
}

fn noise_chunk_vein_interpolator_indexes(
    template: &NoiseChunkTemplate,
) -> NoiseChunkVeinInterpolatorIndexes {
    let lookup_interp = |function: &'static DensityFunction| {
        lookup_density_index(
            &template.interp_by_ptr,
            function as *const DensityFunction as usize,
        )
    };

    NoiseChunkVeinInterpolatorIndexes {
        toggle: lookup_interp(&OVERWORLD_VEIN_TOGGLE_RANGE_DENSITY),
        vein_a: lookup_interp(&OVERWORLD_VEIN_A_RANGE_DENSITY),
        vein_b: lookup_interp(&OVERWORLD_VEIN_B_RANGE_DENSITY),
    }
}

fn assemble_noise_chunk(
    grid: NoiseChunkGrid,
    template: NoiseChunkTemplate,
    allocated: NoiseChunkAllocatedState,
    vein_indexes: NoiseChunkVeinInterpolatorIndexes,
    settings: NoiseGeneratorSettings,
    seed: i64,
    noise_router: NoiseRouter,
) -> NoiseChunk {
    NoiseChunk {
        cell_width: grid.cell_width,
        cell_height: grid.cell_height,
        cell_count_xz: grid.cell_count_xz,
        cell_count_y: grid.cell_count_y,
        cell_noise_min_y: grid.cell_noise_min_y,
        first_cell_x: grid.first_cell_x,
        first_cell_z: grid.first_cell_z,
        noise_size_xz: grid.noise_size_xz,
        interpolators: allocated.interpolators,
        interp_by_ptr: template.interp_by_ptr,
        cache_all_in_cell: allocated.cache_all_in_cell,
        cache_all_by_ptr: template.cache_all_by_ptr,
        cache_2d: RefCell::new(allocated.cache_2d),
        cache_2d_by_ptr: template.cache_2d_by_ptr,
        flat_cache: allocated.flat_cache,
        flat_cache_by_ptr: template.flat_cache_by_ptr,
        full_noise_values: allocated.full_noise_values,
        vein_toggle_values: allocated.vein_toggle_values,
        vein_ridged_values: allocated.vein_ridged_values,
        vein_gap_values: allocated.vein_gap_values,
        vein_toggle_interp_index: vein_indexes.toggle,
        vein_a_interp_index: vein_indexes.vein_a,
        vein_b_interp_index: vein_indexes.vein_b,
        cell_start_block_x: grid.first_cell_x * grid.cell_width,
        cell_start_block_y: grid.cell_noise_min_y * grid.cell_height,
        cell_start_block_z: grid.first_cell_z * grid.cell_width,
        in_cell_x: 0,
        in_cell_y: 0,
        in_cell_z: 0,
        interpolating: false,
        prelim_surface_cache: RefCell::new(HashMap::new()),
        cache_once_values: RefCell::new(allocated.cache_once_values),
        cache_once_by_ptr: template.cache_once_by_ptr,
        normal_noise_cache: RefCell::new(HashMap::new()),
        blended_noise_cache: RefCell::new(Vec::new()),
        density_value_bounds_cache: RefCell::new(HashMap::new()),
        terrain_spline_cache: RefCell::new(HashMap::new()),
        density_array_scratch: Vec::new(),
        filling_cell_cache: Cell::new(false),
        filling_cell: false,
        interpolation_counter: 0,
        array_interpolation_counter: 0,
        array_index: 0,
        fill_stats: NoiseChunkFillStats::default(),
        cache_once_scalar_hits: Cell::new(0),
        cache_once_scalar_misses: Cell::new(0),
        cache_once_array_hits: Cell::new(0),
        cache_once_array_misses: Cell::new(0),
        seed,
        settings,
        noise_router,
    }
}

fn initialize_first_interpolator_slice(mut chunk: NoiseChunk) -> NoiseChunk {
    // Fill slice0 for the first cell-X column (mirrors initializeForFirstCellX).
    chunk.interpolating = true;
    chunk.interpolation_counter = 0;
    chunk.fill_interpolator_slice(true, chunk.first_cell_x * chunk.cell_width);
    chunk
}

impl NoiseChunk {
    /// Create a new `NoiseChunk` for the chunk whose western-most block column
    /// starts at `(chunk_min_block_x, chunk_min_block_z)`.
    ///
    /// Collects all `Interpolated`-marker inner functions from `final_density`
    /// and pre-fills the first X-slice (the slice for `first_cell_x`).
    ///
    /// Mirrors Java `NoiseChunk.forChunk` / `NoiseChunk.initializeForFirstCellX`.
    pub fn new(
        chunk_min_block_x: i32,
        chunk_min_block_z: i32,
        settings: NoiseGeneratorSettings,
        seed: i64,
        noise_router: NoiseRouter,
    ) -> Self {
        let grid = noise_chunk_grid(chunk_min_block_x, chunk_min_block_z, settings);
        let template = noise_chunk_template(settings.id, noise_router);
        let allocated = allocate_noise_chunk_state(
            chunk_min_block_x,
            chunk_min_block_z,
            settings,
            seed,
            &grid,
            &template,
        );
        let vein_indexes = noise_chunk_vein_interpolator_indexes(&template);
        let chunk = assemble_noise_chunk(
            grid,
            template,
            allocated,
            vein_indexes,
            settings,
            seed,
            noise_router,
        );
        initialize_first_interpolator_slice(chunk)
    }

    /// Fill the *next* X-slice (slice1) for `first_cell_x + cell_x_index + 1`
    /// and set `cell_start_block_x` to the start of the current X-cell-column.
    ///
    /// Mirrors Java `NoiseChunk.advanceCellX`.
    pub fn advance_cell_x(&mut self, cell_x_index: i32) {
        let next_cell_x = self.first_cell_x + cell_x_index + 1;
        let block_x = next_cell_x * self.cell_width;
        self.fill_interpolator_slice(false, block_x);
        self.cell_start_block_x = (self.first_cell_x + cell_x_index) * self.cell_width;
        self.interpolating = true;
    }

    fn fill_interpolator_slice(&mut self, use_slice0: bool, block_x: i32) {
        self.cell_start_block_x = block_x;
        self.in_cell_x = 0;
        for z_idx in 0..=(self.cell_count_xz as usize) {
            let block_z = (self.first_cell_z + z_idx as i32) * self.cell_width;
            self.cell_start_block_z = block_z;
            self.in_cell_z = 0;
            self.array_interpolation_counter += 1;

            for interp_index in 0..self.interpolators.len() {
                let inner_fn = self.interpolators[interp_index].inner_fn;
                let mut row = {
                    let interp = &mut self.interpolators[interp_index];
                    if use_slice0 {
                        std::mem::take(&mut interp.slice0[z_idx])
                    } else {
                        std::mem::take(&mut interp.slice1[z_idx])
                    }
                };
                row.resize((self.cell_count_y + 1) as usize, 0.0);
                fill_density_array_with_interp(
                    *inner_fn,
                    self,
                    &mut row,
                    DensityArrayFillMode::Slice { block_x, block_z },
                );
                let interp = &mut self.interpolators[interp_index];
                if use_slice0 {
                    interp.slice0[z_idx] = row;
                } else {
                    interp.slice1[z_idx] = row;
                }
            }
        }
        self.array_interpolation_counter += 1;
    }

    pub(super) fn take_density_array_scratch(&mut self, len: usize) -> Vec<f64> {
        let mut scratch = self
            .density_array_scratch
            .pop()
            .unwrap_or_else(|| Vec::with_capacity(len));
        scratch.resize(len, 0.0);
        scratch
    }

    pub(super) fn return_density_array_scratch(&mut self, mut scratch: Vec<f64>) {
        scratch.clear();
        self.density_array_scratch.push(scratch);
    }

    pub(super) fn density_value_bounds(&self, function: &'static DensityFunction) -> (f64, f64) {
        let key = density_function_key(function);
        if let Some(bounds) = self.density_value_bounds_cache.borrow().get(&key).copied() {
            return bounds;
        }

        let bounds = function.value_bounds();
        self.density_value_bounds_cache
            .borrow_mut()
            .insert(key, bounds);
        bounds
    }

    pub(super) fn normal_noise_sample(
        &self,
        noise_id: &'static str,
        x: f64,
        y: f64,
        z: f64,
    ) -> f64 {
        if !self.normal_noise_cache.borrow().contains_key(noise_id) {
            let Some(snapshot) =
                random_state_normal_noise_snapshot(self.seed, self.settings, noise_id)
            else {
                return 0.0;
            };
            self.normal_noise_cache
                .borrow_mut()
                .insert(noise_id, snapshot);
        }

        self.normal_noise_cache
            .borrow()
            .get(noise_id)
            .map(|snapshot| normal_noise_sample(snapshot, x, y, z))
            .unwrap_or(0.0)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn blended_noise_sample(
        &self,
        xz_scale: f64,
        y_scale: f64,
        xz_factor: f64,
        y_factor: f64,
        smear_scale_multiplier: f64,
        x: f64,
        y: f64,
        z: f64,
    ) -> f64 {
        let key = [
            xz_scale.to_bits(),
            y_scale.to_bits(),
            xz_factor.to_bits(),
            y_factor.to_bits(),
            smear_scale_multiplier.to_bits(),
        ];
        if !self
            .blended_noise_cache
            .borrow()
            .iter()
            .any(|(cached_key, _)| *cached_key == key)
        {
            let Ok(snapshot) = random_state_blended_noise_snapshot(
                self.seed,
                self.settings,
                xz_scale,
                y_scale,
                xz_factor,
                y_factor,
                smear_scale_multiplier,
            ) else {
                return 0.0;
            };
            self.blended_noise_cache.borrow_mut().push((key, snapshot));
        }

        self.blended_noise_cache
            .borrow()
            .iter()
            .find_map(|(cached_key, snapshot)| {
                (*cached_key == key).then(|| blended_noise_sample(snapshot, x, y, z))
            })
            .unwrap_or(0.0)
    }

    pub(super) fn terrain_spline_value(
        &self,
        kind: TerrainSplineKind,
        context: TerrainSplineContext,
    ) -> f64 {
        if let Some(value) = {
            let cache = self.terrain_spline_cache.borrow();
            cache.get(&kind).map(|spline| spline.apply(context))
        } {
            return value;
        }

        let spline = terrain_spline(kind);
        let value = spline.apply(context);
        self.terrain_spline_cache.borrow_mut().insert(kind, spline);
        value
    }

    pub(super) fn interpolator_value(&self, interp_index: usize) -> f64 {
        let interp = &self.interpolators[interp_index];
        if self.filling_cell {
            lerp3(
                f64::from(self.in_cell_x) / f64::from(self.cell_width),
                f64::from(self.in_cell_y) / f64::from(self.cell_height),
                f64::from(self.in_cell_z) / f64::from(self.cell_width),
                interp.noise000,
                interp.noise100,
                interp.noise010,
                interp.noise110,
                interp.noise001,
                interp.noise101,
                interp.noise011,
                interp.noise111,
            )
        } else {
            interp.value
        }
    }

    /// Load the 8 corner values for the (Y, Z) cell at `(cell_y_idx, cell_z_idx)`
    /// and set the block-Y/Z start positions.
    ///
    /// Mirrors Java `NoiseChunk.selectCellYZ`.
    pub fn select_cell_yz(&mut self, cell_y_idx: i32, cell_z_idx: i32) {
        for interp in &mut self.interpolators {
            interp.select_cell_yz(cell_y_idx as usize, cell_z_idx as usize);
        }
        self.cell_start_block_y = (self.cell_noise_min_y + cell_y_idx) * self.cell_height;
        self.cell_start_block_z = (self.first_cell_z + cell_z_idx) * self.cell_width;
        self.filling_cell = true;
        self.filling_cell_cache.set(true);
        self.array_interpolation_counter += 1;
        self.fill_cache_all_in_cell();
        self.fill_full_noise_cache();
        self.array_interpolation_counter += 1;
        self.filling_cell_cache.set(false);
        self.filling_cell = false;
    }

    pub(super) fn cache_all_cell_index(&self) -> Option<usize> {
        let x = self.in_cell_x;
        let y = self.in_cell_y;
        let z = self.in_cell_z;
        (x >= 0
            && y >= 0
            && z >= 0
            && x < self.cell_width
            && y < self.cell_height
            && z < self.cell_width)
            .then_some(
                (((self.cell_height - 1 - y) * self.cell_width + x) * self.cell_width + z) as usize,
            )
    }

    fn fill_cache_all_in_cell(&mut self) {
        if self.cache_all_in_cell.is_empty() {
            return;
        }
        for cache_index in 0..self.cache_all_in_cell.len() {
            let inner_fn = self.cache_all_in_cell[cache_index].inner_fn;
            let mut values = std::mem::take(&mut self.cache_all_in_cell[cache_index].values);
            values.resize(
                (self.cell_width * self.cell_width * self.cell_height) as usize,
                0.0,
            );
            fill_density_array_with_interp(
                *inner_fn,
                self,
                &mut values,
                DensityArrayFillMode::Cell,
            );
            self.cache_all_in_cell[cache_index].values = values;
        }
    }

    pub(super) fn full_noise_uncached_at(&self, x: i32, y: i32, z: i32) -> f64 {
        eval_density_fn_with_interp(self.noise_router.final_density, self, x, y, z)
            + eval_density_fn_with_interp(DensityFunction::Beardifier, self, x, y, z)
    }

    fn fill_full_noise_cache(&mut self) {
        let started = Instant::now();
        let mut values = std::mem::take(&mut self.full_noise_values);
        values.resize(
            (self.cell_width * self.cell_width * self.cell_height) as usize,
            0.0,
        );
        if !self.fill_overworld_final_density_cache(&mut values) {
            fill_density_array_with_interp(
                self.noise_router.final_density,
                self,
                &mut values,
                DensityArrayFillMode::Cell,
            );
        }
        // RustCraft's current Beardifier is a zero stub; keep this cell cache
        // on the same final-density-only path until structure density is wired.
        self.full_noise_values = values;
        let elapsed = started.elapsed();
        self.fill_stats.full_noise_cache_ms += elapsed.as_millis();
        self.fill_stats.full_noise_cache_us += elapsed.as_micros();
        self.fill_stats.full_noise_cache_fills += 1;
    }

    fn fill_overworld_final_density_cache(&mut self, output: &mut [f64]) -> bool {
        if self.settings.id != "minecraft:overworld"
            || self.noise_router.final_density
                != DensityFunction::Reference("minecraft:overworld/final_density")
        {
            return false;
        }

        let blend_ptr = &OVERWORLD_FINAL_BLEND_DENSITY as *const DensityFunction as usize;
        let Some(blend_interp_index) = lookup_density_index(&self.interp_by_ptr, blend_ptr) else {
            return false;
        };
        let noodle_min = OVERWORLD_CAVES_NOODLE_REFERENCE_DENSITY.value_bounds().0;

        let mut needs_noodle = false;
        self.array_index = 0;
        for y_in_cell in (0..self.cell_height).rev() {
            self.in_cell_y = y_in_cell;
            for x_in_cell in 0..self.cell_width {
                self.in_cell_x = x_in_cell;
                for z_in_cell in 0..self.cell_width {
                    self.in_cell_z = z_in_cell;
                    let post_process = MappedDensityFunction::Squeeze
                        .transform(self.interpolator_value(blend_interp_index) * 0.64);
                    if post_process >= noodle_min {
                        needs_noodle = true;
                    }
                    output[self.array_index] = post_process;
                    self.array_index += 1;
                }
            }
        }
        if needs_noodle {
            let mut noodle = self.take_density_array_scratch(output.len());
            if !self.fill_overworld_noodle_density_cache(&mut noodle) {
                fill_density_array_with_interp(
                    OVERWORLD_CAVES_NOODLE_REFERENCE_DENSITY,
                    self,
                    &mut noodle,
                    DensityArrayFillMode::Cell,
                );
            }
            for (value, noodle_value) in output.iter_mut().zip(noodle.iter()) {
                *value = value.min(*noodle_value);
            }
            self.return_density_array_scratch(noodle);
        }
        true
    }

    fn fill_overworld_noodle_density_cache(&mut self, output: &mut [f64]) -> bool {
        let toggle_ptr = &NOODLE_TOGGLE_RANGE_DENSITY as *const DensityFunction as usize;
        let thickness_ptr = &NOODLE_THICKNESS_RANGE_DENSITY as *const DensityFunction as usize;
        let ridge_a_ptr = &NOODLE_RIDGE_A_RANGE_DENSITY as *const DensityFunction as usize;
        let ridge_b_ptr = &NOODLE_RIDGE_B_RANGE_DENSITY as *const DensityFunction as usize;
        let (Some(toggle_index), Some(thickness_index), Some(ridge_a_index), Some(ridge_b_index)) = (
            lookup_density_index(&self.interp_by_ptr, toggle_ptr),
            lookup_density_index(&self.interp_by_ptr, thickness_ptr),
            lookup_density_index(&self.interp_by_ptr, ridge_a_ptr),
            lookup_density_index(&self.interp_by_ptr, ridge_b_ptr),
        ) else {
            return false;
        };

        self.array_index = 0;
        for y_in_cell in (0..self.cell_height).rev() {
            self.in_cell_y = y_in_cell;
            for x_in_cell in 0..self.cell_width {
                self.in_cell_x = x_in_cell;
                for z_in_cell in 0..self.cell_width {
                    self.in_cell_z = z_in_cell;
                    let toggle = self.interpolator_value(toggle_index);
                    output[self.array_index] = if (-1_000_000.0..0.0).contains(&toggle) {
                        64.0
                    } else {
                        let thickness = self.interpolator_value(thickness_index);
                        let ridge_a = self.interpolator_value(ridge_a_index).abs();
                        let ridge_b = self.interpolator_value(ridge_b_index).abs();
                        thickness + 1.5 * ridge_a.max(ridge_b)
                    };
                    self.array_index += 1;
                }
            }
        }
        true
    }

    fn fill_vein_noise_cache(&mut self) {
        let started = Instant::now();
        let len = (self.cell_width * self.cell_width * self.cell_height) as usize;
        let mut toggle = std::mem::take(&mut self.vein_toggle_values);
        toggle.resize(len, 0.0);
        fill_density_array_with_interp(
            self.noise_router.vein_toggle,
            self,
            &mut toggle,
            DensityArrayFillMode::Cell,
        );
        self.vein_toggle_values = toggle;
        let elapsed = started.elapsed();
        self.fill_stats.vein_noise_cache_ms += elapsed.as_millis();
        self.fill_stats.vein_noise_cache_us += elapsed.as_micros();
        self.fill_stats.vein_noise_cache_fills += 1;
    }

    /// Advance the Y factor and update Y-axis partial lerp values for all
    /// interpolators.
    ///
    /// `factor_y = y_in_cell / cell_height` (Java: `(double)posY / cellHeight`).
    ///
    /// Mirrors Java `NoiseChunk.updateForY`.
    pub fn update_for_y(&mut self, pos_y: i32, factor_y: f64) {
        self.in_cell_y = pos_y - self.cell_start_block_y;
        for interp in &mut self.interpolators {
            interp.update_for_y(factor_y);
        }
    }

    /// Advance the X factor and update X-axis partial lerp values for all
    /// interpolators.
    ///
    /// `factor_x = x_in_cell / cell_width`.
    ///
    /// Mirrors Java `NoiseChunk.updateForX`.
    pub fn update_for_x(&mut self, pos_x: i32, factor_x: f64) {
        self.in_cell_x = pos_x - self.cell_start_block_x;
        for interp in &mut self.interpolators {
            interp.update_for_x(factor_x);
        }
    }

    /// Advance the Z factor and write the fully-interpolated `value` for all
    /// interpolators.
    ///
    /// `factor_z = z_in_cell / cell_width`.
    ///
    /// Mirrors Java `NoiseChunk.updateForZ`.
    pub fn update_for_z(&mut self, pos_z: i32, factor_z: f64) {
        self.in_cell_z = pos_z - self.cell_start_block_z;
        self.interpolation_counter += 1;
        for interp in &mut self.interpolators {
            interp.update_for_z(factor_z);
        }
    }

    /// Swap the current and next X-slices for all interpolators.
    ///
    /// Called once per X-cell-column after all blocks in that column have been
    /// processed.  Mirrors Java `NoiseChunk.swapSlices`.
    pub fn swap_slices(&mut self) {
        for interp in &mut self.interpolators {
            interp.swap_slices();
        }
    }

    /// Return the trilinearly-interpolated final density at the current block
    /// position.
    ///
    /// Walks the `final_density` tree, substituting the current interpolated
    /// value for every `Marker(Interpolated)` node tracked by this chunk.
    /// `DensityFunction::Reference` nodes are followed into the built-in
    /// registry so that the overworld's reference-based final_density is
    /// evaluated correctly.
    ///
    /// If there are no `Interpolated` markers in `final_density` (rare), falls
    /// back to a full point evaluation.
    pub fn interpolated_density(&self, x: i32, y: i32, z: i32) -> f64 {
        if let Some(value_index) = self.cache_all_cell_index() {
            if let Some(value) = self.full_noise_values.get(value_index) {
                return *value;
            }
        }

        self.full_noise_uncached_at(x, y, z)
    }

    pub(super) fn cached_vein_toggle(&self, x: i32, y: i32, z: i32) -> f64 {
        if self.noise_router.vein_toggle == OVERWORLD_VEIN_TOGGLE_DENSITY {
            if let Some(index) = self.vein_toggle_interp_index {
                return self.interpolator_value(index);
            }
        }
        eval_density_fn_with_interp(self.noise_router.vein_toggle, self, x, y, z)
    }

    pub(super) fn vein_ridged_at(&self, x: i32, y: i32, z: i32) -> f64 {
        if self.noise_router.vein_ridged == OVERWORLD_VEIN_RIDGED_DENSITY {
            if let (Some(a_index), Some(b_index)) =
                (self.vein_a_interp_index, self.vein_b_interp_index)
            {
                return -0.079_999_998_211_860_66
                    + self
                        .interpolator_value(a_index)
                        .abs()
                        .max(self.interpolator_value(b_index).abs());
            }
        }
        eval_density_fn_with_interp(self.noise_router.vein_ridged, self, x, y, z)
    }

    pub(super) fn vein_gap_at(&self, x: i32, y: i32, z: i32) -> f64 {
        eval_density_fn_with_interp(self.noise_router.vein_gap, self, x, y, z)
    }

    pub(super) fn take_fill_stats(&mut self) -> NoiseChunkFillStats {
        let mut stats = self.fill_stats;
        stats.cache_once_scalar_hits = self.cache_once_scalar_hits.replace(0);
        stats.cache_once_scalar_misses = self.cache_once_scalar_misses.replace(0);
        stats.cache_once_array_hits = self.cache_once_array_hits.replace(0);
        stats.cache_once_array_misses = self.cache_once_array_misses.replace(0);
        self.fill_stats = NoiseChunkFillStats::default();
        stats
    }

    /// Compute and cache the preliminary surface level at the given block
    /// column, quantised to the nearest quart-block boundary (matching Java's
    /// `QuartPos.toBlock(QuartPos.fromBlock(x))`).
    ///
    /// Mirrors Java `NoiseChunk.preliminarySurfaceLevel`.
    pub fn preliminary_surface_level(&self, block_x: i32, block_z: i32) -> i32 {
        // Quantise: shift right by 2 then left by 2 (= round down to multiple of 4).
        let qx = (block_x >> 2) << 2;
        let qz = (block_z >> 2) << 2;
        let key = pack_column(qx, qz);
        if let Some(cached) = self.prelim_surface_cache.borrow().get(&key).copied() {
            return cached;
        }
        let value = eval_density_fn_single_point(
            self.noise_router.preliminary_surface_level,
            self,
            qx,
            0,
            qz,
        )
        .floor() as i32;
        self.prelim_surface_cache.borrow_mut().insert(key, value);
        value
    }

    pub fn max_preliminary_surface_level(
        &self,
        min_block_x: i32,
        min_block_z: i32,
        max_block_x: i32,
        max_block_z: i32,
    ) -> i32 {
        let mut max_y = i32::MIN;
        let mut block_z = min_block_z;
        while block_z <= max_block_z {
            let mut block_x = min_block_x;
            while block_x <= max_block_x {
                max_y = max_y.max(self.preliminary_surface_level(block_x, block_z));
                block_x += 4;
            }
            block_z += 4;
        }
        max_y
    }
}
