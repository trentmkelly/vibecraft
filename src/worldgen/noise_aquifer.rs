use super::*;

impl FluidStatus {
    pub fn at(self, block_y: i32) -> &'static str {
        if block_y < self.fluid_level {
            self.fluid_type
        } else {
            "minecraft:air"
        }
    }
}

pub fn disabled_aquifer_substance(
    density: f64,
    fluid: FluidStatus,
    block_y: i32,
) -> Option<&'static str> {
    if density > 0.0 {
        None
    } else {
        Some(fluid.at(block_y))
    }
}

pub fn aquifer_similarity(distance_sqr_1: i32, distance_sqr_2: i32) -> f64 {
    1.0 - f64::from((distance_sqr_2 - distance_sqr_1).abs()) / 25.0
}

// ── Noise-Based Aquifer ────────────────────────────────────────────────────────
//
// Mirrors Java `Aquifer.NoiseBasedAquifer`.  Places underground water / lava
// pockets using a Voronoi-cell grid with barrier-noise pressure between cells.

/// Sentinel: far below any reachable Y, used when an aquifer cell has no fluid.
/// Java: `DimensionType.WAY_BELOW_MIN_Y = MIN_Y << 4 = -2032 * 16 = -32512`.
const AQUIFER_WAY_BELOW_MIN_Y: i32 = -32512;

/// FLOWING_UPDATE_SIMILARITY = similarity(10², 12²) = 1 − (144−100)/25 = −0.76.
const AQUIFER_FLOWING_UPDATE_SIMILARITY: f64 = -0.76;

/// Returns the global (non-aquifer) fluid status at the given Y level.
///
/// Mirrors Java `NoiseBasedChunkGenerator.createFluidPicker`:
///   - `y < min(-54, seaLevel)` → lava with `fluid_level=-54`
///   - otherwise               → sea-level water / default fluid
pub(super) fn global_fluid_status(
    y: i32,
    sea_level: i32,
    default_fluid: &'static str,
) -> FluidStatus {
    const LAVA_Y: i32 = -54;
    if y < LAVA_Y.min(sea_level) {
        FluidStatus {
            fluid_level: LAVA_Y,
            fluid_type: "minecraft:lava",
        }
    } else {
        FluidStatus {
            fluid_level: sea_level,
            fluid_type: default_fluid,
        }
    }
}

/// Grid coordinate helpers — mirrors Java `NoiseBasedAquifer` private statics.
#[inline]
fn aq_grid_x(block: i32) -> i32 {
    block >> 4
}
#[inline]
fn aq_grid_y(block: i32) -> i32 {
    block.div_euclid(12)
}
#[inline]
fn aq_grid_z(block: i32) -> i32 {
    block >> 4
}
#[inline]
fn aq_from_grid_x(grid: i32, off: i32) -> i32 {
    (grid << 4) + off
}
#[inline]
fn aq_from_grid_y(grid: i32, off: i32) -> i32 {
    grid * 12 + off
}
#[inline]
fn aq_from_grid_z(grid: i32, off: i32) -> i32 {
    (grid << 4) + off
}

/// Pack (x, y, z) into the same 64-bit layout as Java `BlockPos.asLong`.
#[inline]
fn aq_pack_pos(x: i32, y: i32, z: i32) -> i64 {
    (((x as i64) & 0x3ff_ffff) << 38) | (((z as i64) & 0x3ff_ffff) << 12) | ((y as i64) & 0xfff)
}

/// Extract X from a packed `BlockPos.asLong` value (arithmetic right-shift).
#[inline]
fn aq_unpack_x(p: i64) -> i32 {
    (p >> 38) as i32
}
/// Extract Z.
#[inline]
fn aq_unpack_z(p: i64) -> i32 {
    ((p << 26) >> 38) as i32
}
/// Extract Y.
#[inline]
fn aq_unpack_y(p: i64) -> i32 {
    ((p << 52) >> 52) as i32
}

/// Private `similarity` as in Java — no abs (d1 ≤ d2 guaranteed by caller).
#[inline]
fn aq_similarity(d1: i32, d2: i32) -> f64 {
    1.0 - (d2 - d1) as f64 / 25.0
}

/// Preliminary surface level at (x, z) evaluated from the noise router.
/// Rounds down to QuartPos resolution (multiple of 4) to match Java.
fn aq_prelim_surface(
    x: i32,
    z: i32,
    seed: i64,
    settings: NoiseGeneratorSettings,
    noise_router: NoiseRouter,
) -> i32 {
    let qx = (x >> 2) << 2;
    let qz = (z >> 2) << 2;
    noise_router
        .preliminary_surface_level
        .compute_with_noise(seed, settings, qx, 0, qz)
        .floor() as i32
}

/// Whether (x, y, z) is in the Deep Dark climate region.
/// Mirrors Java `OverworldBiomeBuilder.isDeepDarkRegion`.
fn aq_is_deep_dark(pos: BlockPos, context: AquiferNoiseContext<'_>) -> bool {
    // Java uses float literals (-0.225F, 0.9F), widened to double for comparison.
    let erosion = if let Some(chunk) = context.noise_chunk {
        eval_density_fn_with_interp(context.noise_router.erosion, chunk, pos.x, pos.y, pos.z)
    } else {
        context.noise_router.erosion.compute_with_noise(
            context.seed,
            context.settings,
            pos.x,
            pos.y,
            pos.z,
        )
    };
    let depth = if let Some(chunk) = context.noise_chunk {
        eval_density_fn_with_interp(context.noise_router.depth, chunk, pos.x, pos.y, pos.z)
    } else {
        context.noise_router.depth.compute_with_noise(
            context.seed,
            context.settings,
            pos.x,
            pos.y,
            pos.z,
        )
    };
    erosion < (-0.225_f32) as f64 && depth > (0.9_f32) as f64
}

#[derive(Clone, Copy)]
struct AquiferNoiseContext<'a> {
    seed: i64,
    settings: NoiseGeneratorSettings,
    noise_router: NoiseRouter,
    noise_chunk: Option<&'a NoiseChunk>,
}

/// Calculate the barrier pressure between two adjacent aquifer cells.
///
/// Mirrors Java `NoiseBasedAquifer.calculatePressure`.
fn aq_calculate_pressure(
    pos: BlockPos,
    context: AquiferNoiseContext<'_>,
    barrier_cache: &mut Option<f64>,
    s1: &FluidStatus,
    s2: &FluidStatus,
) -> f64 {
    let t1 = s1.at(pos.y);
    let t2 = s2.at(pos.y);

    // Water/lava interface → maximum pressure (always solid barrier).
    if (t1 == "minecraft:lava" && t2 == "minecraft:water")
        || (t1 == "minecraft:water" && t2 == "minecraft:lava")
    {
        return 2.0;
    }

    let diff = (s1.fluid_level - s2.fluid_level).abs();
    if diff == 0 {
        return 0.0;
    }

    let avg_y = 0.5 * (s1.fluid_level + s2.fluid_level) as f64;
    let above = pos.y as f64 + 0.5 - avg_y;
    let base = diff as f64 / 2.0;
    let dist_from_edge = base - above.abs();

    let gradient = if above > 0.0 {
        if dist_from_edge > 0.0 {
            dist_from_edge / 1.5
        } else {
            dist_from_edge / 2.5
        }
    } else {
        let v = 3.0 + dist_from_edge;
        if v > 0.0 {
            v / 3.0
        } else {
            v / 10.0
        }
    };

    // Only sample barrier noise when gradient is in the influential range.
    let noise_val = if (-2.0..=2.0).contains(&gradient) {
        *barrier_cache.get_or_insert_with(|| {
            if let Some(chunk) = context.noise_chunk {
                eval_density_fn_with_interp(
                    context.noise_router.barrier,
                    chunk,
                    pos.x,
                    pos.y,
                    pos.z,
                )
            } else {
                context.noise_router.barrier.compute_with_noise(
                    context.seed,
                    context.settings,
                    pos.x,
                    pos.y,
                    pos.z,
                )
            }
        })
    } else {
        0.0
    };

    2.0 * (noise_val + gradient)
}

/// Compute the randomised fluid surface level for a partially-flooded cell.
///
/// Mirrors Java `NoiseBasedAquifer.computeRandomizedFluidSurfaceLevel`.
fn aq_randomized_fluid_level(
    pos: BlockPos,
    lowest_prelim: i32,
    context: AquiferNoiseContext<'_>,
) -> i32 {
    const CW: i32 = 16; // fluidCellWidth
    const CH: i32 = 40; // fluidCellHeight
    let cx = pos.x.div_euclid(CW);
    let cy = pos.y.div_euclid(CH);
    let cz = pos.z.div_euclid(CW);
    let mid_y = cy * CH + 20;
    let spread = if let Some(chunk) = context.noise_chunk {
        eval_density_fn_with_interp(context.noise_router.fluid_level_spread, chunk, cx, cy, cz)
    } else {
        context.noise_router.fluid_level_spread.compute_with_noise(
            context.seed,
            context.settings,
            cx,
            cy,
            cz,
        )
    } * 10.0;
    // Mth.quantize(v, 3) = floor(v / 3) * 3
    let quantized = (spread / 3.0).floor() as i32 * 3;
    (mid_y + quantized).min(lowest_prelim)
}

/// Compute the fluid type (water or lava) for an aquifer cell.
///
/// Mirrors Java `NoiseBasedAquifer.computeFluidType`.
fn aq_fluid_type(
    pos: BlockPos,
    global_fluid: &FluidStatus,
    fluid_level: i32,
    context: AquiferNoiseContext<'_>,
) -> &'static str {
    // Lava pockets only appear well below sea level and when lava noise is strong.
    if fluid_level <= -10
        && fluid_level != AQUIFER_WAY_BELOW_MIN_Y
        && global_fluid.fluid_type != "minecraft:lava"
    {
        const LCW: i32 = 64; // lava cell width
        const LCH: i32 = 40; // lava cell height
        let cx = pos.x.div_euclid(LCW);
        let cy = pos.y.div_euclid(LCH);
        let cz = pos.z.div_euclid(LCW);
        let val = if let Some(chunk) = context.noise_chunk {
            eval_density_fn_with_interp(context.noise_router.lava, chunk, cx, cy, cz)
        } else {
            context
                .noise_router
                .lava
                .compute_with_noise(context.seed, context.settings, cx, cy, cz)
        };
        if val.abs() > 0.3 {
            return "minecraft:lava";
        }
    }
    global_fluid.fluid_type
}

/// Compute the fluid surface level for an aquifer cell given its neighbourhood context.
///
/// Mirrors Java `NoiseBasedAquifer.computeSurfaceLevel`.
fn aq_surface_level(
    pos: BlockPos,
    global_fluid: &FluidStatus,
    lowest_prelim: i32,
    surface_at_center_under_fluid: bool,
    context: AquiferNoiseContext<'_>,
) -> i32 {
    let (partially_flooded, fully_flooded) = if aq_is_deep_dark(pos, context) {
        // Deep dark: no aquifer pockets.
        (-1.0_f64, -1.0_f64)
    } else {
        let dist_below = (lowest_prelim + 8) - pos.y;
        // Mth.clampedMap(dist_below, 0, 64, 1, 0) = clamp(1 - dist_below/64, 0, 1)
        let flooded_factor = if surface_at_center_under_fluid {
            (1.0 - dist_below as f64 / 64.0).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let noise = if let Some(chunk) = context.noise_chunk {
            eval_density_fn_with_interp(
                context.noise_router.fluid_level_floodedness,
                chunk,
                pos.x,
                pos.y,
                pos.z,
            )
        } else {
            context
                .noise_router
                .fluid_level_floodedness
                .compute_with_noise(context.seed, context.settings, pos.x, pos.y, pos.z)
        }
        .clamp(-1.0, 1.0);
        // Mth.map(flooded_factor, 1.0, 0.0, -0.3, 0.8) = -0.3 + (1-f)*1.1
        let fully_threshold = -0.3 + (1.0 - flooded_factor) * 1.1;
        // Mth.map(flooded_factor, 1.0, 0.0, -0.8, 0.4) = -0.8 + (1-f)*1.2
        let partial_threshold = -0.8 + (1.0 - flooded_factor) * 1.2;
        (noise - partial_threshold, noise - fully_threshold)
    };

    if fully_flooded > 0.0 {
        global_fluid.fluid_level
    } else if partially_flooded > 0.0 {
        aq_randomized_fluid_level(pos, lowest_prelim, context)
    } else {
        AQUIFER_WAY_BELOW_MIN_Y
    }
}

/// Compute the complete `FluidStatus` for an aquifer cell at (x, y, z).
///
/// Mirrors Java `NoiseBasedAquifer.computeFluid`.
fn aq_compute_fluid(pos: BlockPos, context: AquiferNoiseContext<'_>) -> FluidStatus {
    let global = global_fluid_status(
        pos.y,
        context.settings.sea_level,
        context.settings.default_fluid,
    );
    let top = pos.y + 12;
    let bottom = pos.y - 12;
    let mut lowest_prelim = i32::MAX;
    let mut center_under_fluid = false;

    // 13 chunk offsets around the aquifer cell — mirrors Java SURFACE_SAMPLING_OFFSETS_IN_CHUNKS.
    // Each offset is in chunk units (× 16 blocks).
    const OFFSETS: [(i32, i32); 13] = [
        (0, 0),
        (-2, -1),
        (-1, -1),
        (0, -1),
        (1, -1),
        (-3, 0),
        (-2, 0),
        (-1, 0),
        (1, 0),
        (-2, 1),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    for (ox, oz) in OFFSETS {
        let sx = pos.x + ox * 16;
        let sz = pos.z + oz * 16;
        let prelim = context.noise_chunk.map_or_else(
            || aq_prelim_surface(sx, sz, context.seed, context.settings, context.noise_router),
            |chunk| chunk.preliminary_surface_level(sx, sz),
        );
        // adjustSurfaceLevel adds 8 to widen the "near surface" check.
        let adjusted = prelim + 8;
        let is_center = ox == 0 && oz == 0;

        if is_center && bottom > adjusted {
            // Aquifer cell is entirely above the terrain surface → use global fluid.
            return global;
        }

        let pokes_above = top > adjusted;
        if pokes_above || is_center {
            // Check if the surface at this offset is under global fluid (e.g., ocean floor).
            let surf_fluid = global_fluid_status(
                adjusted,
                context.settings.sea_level,
                context.settings.default_fluid,
            );
            if surf_fluid.at(adjusted) != "minecraft:air" {
                if is_center {
                    center_under_fluid = true;
                }
                if pokes_above {
                    // Cell top pierces the adjusted surface under fluid → use global fluid.
                    return surf_fluid;
                }
            }
        }
        lowest_prelim = lowest_prelim.min(prelim);
    }

    let fluid_level = aq_surface_level(pos, &global, lowest_prelim, center_under_fluid, context);
    let fluid_type = aq_fluid_type(pos, &global, fluid_level, context);
    FluidStatus {
        fluid_level,
        fluid_type,
    }
}

/// Per-chunk noise-based aquifer.  Placed between `NoiseBasedAquifer::new` and
/// `compute_substance` calls in the block-fill loop of `fill_from_noise_chunk_inner`.
pub(super) struct NoiseBasedAquifer {
    seed: i64,
    settings: NoiseGeneratorSettings,
    noise_router: NoiseRouter,
    pos_factory: crate::random_source::PositionalRandomFactory,
    min_grid_x: i32,
    min_grid_y: i32,
    min_grid_z: i32,
    grid_size_x: i32,
    grid_size_z: i32,
    /// Y above which we skip aquifer calculation (handled by the fast-path global picker).
    skip_sampling_above_y: i32,
    /// Cached `FluidStatus` per aquifer grid cell (`None` = not yet computed).
    aquifer_cache: Vec<Option<FluidStatus>>,
    /// Packed BlockPos of the random centre of each aquifer cell (`i64::MAX` = not sampled).
    aquifer_location_cache: Vec<i64>,
    pub should_schedule_fluid_update: bool,
}

#[derive(Clone, Copy)]
pub(super) struct NoiseBasedAquiferBounds {
    pub chunk_min_x: i32,
    pub chunk_max_x: i32,
    pub chunk_min_z: i32,
    pub chunk_max_z: i32,
    pub min_block_y: i32,
    pub y_block_size: i32,
}

#[derive(Clone, Copy)]
struct NearestAquiferCells {
    distances: [i32; 4],
    indices: [usize; 4],
}

impl NearestAquiferCells {
    fn new() -> Self {
        Self {
            distances: [i32::MAX; 4],
            indices: [0; 4],
        }
    }

    fn insert(&mut self, index: usize, distance: i32) {
        for slot in 0..4 {
            if distance > self.distances[slot] {
                continue;
            }
            for move_slot in ((slot + 1)..4).rev() {
                self.distances[move_slot] = self.distances[move_slot - 1];
                self.indices[move_slot] = self.indices[move_slot - 1];
            }
            self.distances[slot] = distance;
            self.indices[slot] = index;
            break;
        }
    }
}

#[derive(Clone, Copy)]
struct AquiferPressureEvaluation {
    s2: FluidStatus,
    s3: FluidStatus,
    sim13: f64,
    sim23: f64,
}

impl NoiseBasedAquifer {
    pub(super) fn new(
        noise_chunk: &mut NoiseChunk,
        bounds: NoiseBasedAquiferBounds,
        seed: i64,
        settings: NoiseGeneratorSettings,
        noise_router: NoiseRouter,
        pos_factory: crate::random_source::PositionalRandomFactory,
    ) -> Self {
        // Java: SAMPLE_OFFSET_X = SAMPLE_OFFSET_Z = -5
        const OFF: i32 = -5;

        let min_grid_x = aq_grid_x(bounds.chunk_min_x + OFF);
        let max_grid_x = aq_grid_x(bounds.chunk_max_x + OFF) + 1;
        let grid_size_x = max_grid_x - min_grid_x + 1;

        let min_grid_y = aq_grid_y(bounds.min_block_y + 1) - 1;
        let max_grid_y = aq_grid_y(bounds.min_block_y + bounds.y_block_size + 1) + 1;
        let grid_size_y = max_grid_y - min_grid_y + 1;

        let min_grid_z = aq_grid_z(bounds.chunk_min_z + OFF);
        let max_grid_z = aq_grid_z(bounds.chunk_max_z + OFF) + 1;
        let grid_size_z = max_grid_z - min_grid_z + 1;

        let total = (grid_size_x * grid_size_y * grid_size_z) as usize;
        let aquifer_cache = vec![None; total];
        let aquifer_location_cache = vec![i64::MAX; total];

        // Java Aquifer.NoiseBasedAquifer asks NoiseChunk for the maximum
        // preliminary surface over the whole aquifer grid, sampled every 4
        // blocks.  This controls how much vertical range needs expensive
        // aquifer sampling.
        let max_prelim = noise_chunk.max_preliminary_surface_level(
            aq_from_grid_x(min_grid_x, 0),
            aq_from_grid_z(min_grid_z, 0),
            aq_from_grid_x(max_grid_x, 9),
            aq_from_grid_z(max_grid_z, 9),
        );
        // adjustSurfaceLevel(prelim) = prelim + 8
        let max_adjusted = max_prelim + 8;
        // skipSamplingAboveGridY = gridY(max_adjusted + 12) - (-1) = gridY(...) + 1
        let skip_grid_y = aq_grid_y(max_adjusted + 12) + 1;
        // fromGridY(skipGridY, 11) - 1
        let skip_sampling_above_y = aq_from_grid_y(skip_grid_y, 11) - 1;

        NoiseBasedAquifer {
            seed,
            settings,
            noise_router,
            pos_factory,
            min_grid_x,
            min_grid_y,
            min_grid_z,
            grid_size_x,
            grid_size_z,
            skip_sampling_above_y,
            aquifer_cache,
            aquifer_location_cache,
            should_schedule_fluid_update: false,
        }
    }

    #[inline]
    fn get_index(&self, gx: i32, gy: i32, gz: i32) -> usize {
        let x = (gx - self.min_grid_x) as usize;
        let y = (gy - self.min_grid_y) as usize;
        let z = (gz - self.min_grid_z) as usize;
        (y * self.grid_size_z as usize + z) * self.grid_size_x as usize + x
    }

    /// Compute the substance (block ID) for a position with negative density.
    ///
    /// Returns `None` when barrier pressure converts the empty space to solid stone.
    /// Returns `Some(block_id)` for the fluid or air that fills the position.
    ///
    /// Mirrors Java `NoiseBasedAquifer.computeSubstance`.
    pub fn compute_substance(
        &mut self,
        noise_chunk: &NoiseChunk,
        pos_x: i32,
        pos_y: i32,
        pos_z: i32,
        density: f64,
    ) -> Option<&'static str> {
        let pos = BlockPos {
            x: pos_x,
            y: pos_y,
            z: pos_z,
        };

        if density > 0.0 {
            self.should_schedule_fluid_update = false;
            return None; // solid
        }

        let global =
            global_fluid_status(pos_y, self.settings.sea_level, self.settings.default_fluid);

        // Fast path: above the preliminary surface, just use the global picker.
        if pos_y > self.skip_sampling_above_y {
            self.should_schedule_fluid_update = false;
            return Some(global.at(pos_y));
        }

        // Near-bedrock lava: return immediately without Voronoi calculation.
        if global.at(pos_y) == "minecraft:lava" {
            self.should_schedule_fluid_update = false;
            return Some("minecraft:lava");
        }

        let nearest = self.nearest_aquifer_cells(pos);
        let s1 = self.get_aquifer_status(nearest.indices[0], noise_chunk);
        let sim12 = aq_similarity(nearest.distances[0], nearest.distances[1]);
        let fluid1 = s1.at(pos_y);

        if sim12 <= 0.0 {
            self.update_flow_schedule_for_source_pair(sim12, s1, nearest.indices[1], noise_chunk);
            return Some(fluid1);
        }

        // Water touching lava at the block below → schedule update.
        if self.water_touches_lava_below(fluid1, pos_y) {
            self.should_schedule_fluid_update = true;
            return Some(fluid1);
        }

        let pressure =
            self.evaluate_barrier_pressures(noise_chunk, pos, density, nearest, s1, sim12)?;
        self.update_flow_schedule_after_pressure(noise_chunk, nearest, s1, pressure);
        Some(fluid1)
    }

    fn nearest_aquifer_cells(&mut self, pos: BlockPos) -> NearestAquiferCells {
        let x_anc = aq_grid_x(pos.x - 5);
        let y_anc = aq_grid_y(pos.y + 1);
        let z_anc = aq_grid_z(pos.z - 5);
        let mut nearest = NearestAquiferCells::new();

        for dx in 0..=1_i32 {
            for dy in -1..=1_i32 {
                for dz in 0..=1_i32 {
                    let gx = x_anc + dx;
                    let gy = y_anc + dy;
                    let gz = z_anc + dz;
                    let idx = self.get_index(gx, gy, gz);

                    let cached = self.aquifer_location_cache[idx];
                    let loc = if cached != i64::MAX {
                        cached
                    } else {
                        let mut rng = self.pos_factory.at(gx, gy, gz);
                        let l = aq_pack_pos(
                            aq_from_grid_x(gx, random_next_i32_bound(&mut rng, 10)),
                            aq_from_grid_y(gy, random_next_i32_bound(&mut rng, 9)),
                            aq_from_grid_z(gz, random_next_i32_bound(&mut rng, 10)),
                        );
                        self.aquifer_location_cache[idx] = l;
                        l
                    };

                    let cx = aq_unpack_x(loc) - pos.x;
                    let cy = aq_unpack_y(loc) - pos.y;
                    let cz = aq_unpack_z(loc) - pos.z;
                    let dist = cx * cx + cy * cy + cz * cz;
                    nearest.insert(idx, dist);
                }
            }
        }
        nearest
    }

    fn update_flow_schedule_for_source_pair(
        &mut self,
        similarity: f64,
        source: FluidStatus,
        second_index: usize,
        noise_chunk: &NoiseChunk,
    ) {
        if similarity >= AQUIFER_FLOWING_UPDATE_SIMILARITY {
            let s2 = self.get_aquifer_status(second_index, noise_chunk);
            self.should_schedule_fluid_update = source != s2;
        } else {
            self.should_schedule_fluid_update = false;
        }
    }

    fn water_touches_lava_below(&self, fluid: &str, pos_y: i32) -> bool {
        if fluid == "minecraft:water" {
            let below = global_fluid_status(
                pos_y - 1,
                self.settings.sea_level,
                self.settings.default_fluid,
            );
            return below.at(pos_y - 1) == "minecraft:lava";
        }
        false
    }

    fn evaluate_barrier_pressures(
        &mut self,
        noise_chunk: &NoiseChunk,
        pos: BlockPos,
        density: f64,
        nearest: NearestAquiferCells,
        s1: FluidStatus,
        sim12: f64,
    ) -> Option<AquiferPressureEvaluation> {
        let context = AquiferNoiseContext {
            seed: self.seed,
            settings: self.settings,
            noise_router: self.noise_router,
            noise_chunk: Some(noise_chunk),
        };
        let mut barrier: Option<f64> = None;
        let s2 = self.get_aquifer_status(nearest.indices[1], noise_chunk);
        let p12 = sim12 * aq_calculate_pressure(pos, context, &mut barrier, &s1, &s2);
        if density + p12 > 0.0 {
            self.should_schedule_fluid_update = false;
            return None;
        }

        let s3 = self.get_aquifer_status(nearest.indices[2], noise_chunk);
        let sim13 = aq_similarity(nearest.distances[0], nearest.distances[2]);
        if sim13 > 0.0 {
            let p13 = sim12 * sim13 * aq_calculate_pressure(pos, context, &mut barrier, &s1, &s3);
            if density + p13 > 0.0 {
                self.should_schedule_fluid_update = false;
                return None;
            }
        }

        let sim23 = aq_similarity(nearest.distances[1], nearest.distances[2]);
        if sim23 > 0.0 {
            let p23 = sim12 * sim23 * aq_calculate_pressure(pos, context, &mut barrier, &s2, &s3);
            if density + p23 > 0.0 {
                self.should_schedule_fluid_update = false;
                return None;
            }
        }

        Some(AquiferPressureEvaluation {
            s2,
            s3,
            sim13,
            sim23,
        })
    }

    fn update_flow_schedule_after_pressure(
        &mut self,
        noise_chunk: &NoiseChunk,
        nearest: NearestAquiferCells,
        s1: FluidStatus,
        pressure: AquiferPressureEvaluation,
    ) {
        let flow12 = s1 != pressure.s2;
        let flow23 =
            pressure.sim23 >= AQUIFER_FLOWING_UPDATE_SIMILARITY && pressure.s2 != pressure.s3;
        let flow13 = pressure.sim13 >= AQUIFER_FLOWING_UPDATE_SIMILARITY && s1 != pressure.s3;
        if !flow12 && !flow23 && !flow13 {
            let sim14 = aq_similarity(nearest.distances[0], nearest.distances[3]);
            let s4 = self.get_aquifer_status(nearest.indices[3], noise_chunk);
            self.should_schedule_fluid_update = pressure.sim13 >= AQUIFER_FLOWING_UPDATE_SIMILARITY
                && sim14 >= AQUIFER_FLOWING_UPDATE_SIMILARITY
                && s1 != s4;
        } else {
            self.should_schedule_fluid_update = true;
        }
    }

    fn get_aquifer_status(&mut self, index: usize, noise_chunk: &NoiseChunk) -> FluidStatus {
        if let Some(status) = self.aquifer_cache[index] {
            return status;
        }
        let loc = self.aquifer_location_cache[index];
        let x = aq_unpack_x(loc);
        let y = aq_unpack_y(loc);
        let z = aq_unpack_z(loc);
        let status = aq_compute_fluid(
            BlockPos { x, y, z },
            AquiferNoiseContext {
                seed: self.seed,
                settings: self.settings,
                noise_router: self.noise_router,
                noise_chunk: Some(noise_chunk),
            },
        );
        self.aquifer_cache[index] = Some(status);
        status
    }
}
