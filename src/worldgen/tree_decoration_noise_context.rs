fn tree_context_worker_count(context_count: usize) -> usize {
    let available = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    let default_workers = available.min(8).min(context_count).max(1);
    std::env::var("VIBECRAFT_WORLDGEN_TREE_CONTEXT_THREADS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .map(|requested| requested.clamp(1, context_count.max(1)))
        .unwrap_or(default_workers)
}

pub(super) fn tree_decoration_source_radius() -> i32 {
    std::env::var("VIBECRAFT_WORLDGEN_TREE_SOURCE_RADIUS")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(1)
        .clamp(0, 1)
}

pub(super) fn noise_tree_context_heights(
    pos: ChunkPos,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    noise_router: NoiseRouter,
) -> TreeDecorationHeights {
    with_noise_snapshot_cache(|| {
        noise_tree_context_heights_inner(pos, settings, seed, noise_router)
    })
}

#[derive(Debug, Clone, Copy)]
struct NoiseTreeContextGeometry {
    min_y: i32,
    height: i32,
    chunk_min_x: i32,
    chunk_min_z: i32,
    cell_width: i32,
    cell_height: i32,
    cell_count_xz: i32,
    cell_noise_min_y: i32,
}

impl NoiseTreeContextGeometry {
    fn new(pos: ChunkPos, settings: &NoiseGeneratorSettings) -> Self {
        let min_y = settings.noise.min_y;
        let cell_height = settings.noise.cell_height();
        Self {
            min_y,
            height: settings.noise.height,
            chunk_min_x: pos.x * 16,
            chunk_min_z: pos.z * 16,
            cell_width: settings.noise.cell_width(),
            cell_height,
            cell_count_xz: 16 / settings.noise.cell_width(),
            cell_noise_min_y: min_y.div_euclid(cell_height),
        }
    }
}

struct NoiseTreeContextScanTops {
    by_column: [i32; 16 * 16],
    max_cell_y: i32,
}

struct NoiseTreeContextHeightmaps {
    ocean_floor: [i32; 16 * 16],
    world_surface: [i32; 16 * 16],
    motion_blocking: [i32; 16 * 16],
    motion_blocking_no_leaves: [i32; 16 * 16],
    found_ocean_floor: [bool; 16 * 16],
    found_world_surface: [bool; 16 * 16],
    remaining_ocean_floor: usize,
    remaining_world_surface: usize,
}

impl NoiseTreeContextHeightmaps {
    fn new(min_y: i32) -> Self {
        Self {
            ocean_floor: [min_y; 16 * 16],
            world_surface: [min_y; 16 * 16],
            motion_blocking: [min_y; 16 * 16],
            motion_blocking_no_leaves: [min_y; 16 * 16],
            found_ocean_floor: [false; 16 * 16],
            found_world_surface: [false; 16 * 16],
            remaining_ocean_floor: 16 * 16,
            remaining_world_surface: 16 * 16,
        }
    }

    fn is_complete_at(&self, index: usize) -> bool {
        self.found_ocean_floor[index] && self.found_world_surface[index]
    }

    fn record_block(&mut self, index: usize, y: i32, block_kind: NoiseHeightmapBlockKind) -> bool {
        if !self.found_world_surface[index] {
            self.world_surface[index] = y + 1;
            self.motion_blocking[index] = y + 1;
            self.motion_blocking_no_leaves[index] = y + 1;
            self.found_world_surface[index] = true;
            self.remaining_world_surface -= 1;
        }
        if block_kind == NoiseHeightmapBlockKind::Solid {
            self.ocean_floor[index] = y + 1;
            self.found_ocean_floor[index] = true;
            self.remaining_ocean_floor -= 1;
        }
        self.remaining_world_surface == 0 && self.remaining_ocean_floor == 0
    }

    fn into_tree_decoration_heights(self) -> TreeDecorationHeights {
        TreeDecorationHeights {
            ocean_floor: self.ocean_floor,
            world_surface: self.world_surface,
            motion_blocking: self.motion_blocking,
            motion_blocking_no_leaves: self.motion_blocking_no_leaves,
        }
    }
}

struct NoiseTreeContextHeightStats {
    density_samples: usize,
    fluid_samples: usize,
    lowest_sampled_y: i32,
    highest_sampled_y: i32,
}

impl Default for NoiseTreeContextHeightStats {
    fn default() -> Self {
        Self {
            density_samples: 0,
            fluid_samples: 0,
            lowest_sampled_y: i32::MAX,
            highest_sampled_y: i32::MIN,
        }
    }
}

fn noise_tree_context_heights_inner(
    pos: ChunkPos,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    noise_router: NoiseRouter,
) -> TreeDecorationHeights {
    let geometry = NoiseTreeContextGeometry::new(pos, settings);
    let mut noise_chunk = NoiseChunk::new(
        geometry.chunk_min_x,
        geometry.chunk_min_z,
        *settings,
        seed,
        noise_router,
    );
    let algorithm = if settings.legacy_random_source {
        crate::random_source::RandomAlgorithm::Legacy
    } else {
        crate::random_source::RandomAlgorithm::Xoroshiro
    };
    let factories = crate::random_source::random_state_seed_factories(seed, algorithm);
    let mut aquifer = settings.aquifers_enabled.then(|| {
        NoiseBasedAquifer::new(
            &mut noise_chunk,
            NoiseBasedAquiferBounds {
                chunk_min_x: geometry.chunk_min_x,
                chunk_max_x: geometry.chunk_min_x + 15,
                chunk_min_z: geometry.chunk_min_z,
                chunk_max_z: geometry.chunk_min_z + 15,
                min_block_y: geometry.min_y,
                y_block_size: geometry.height,
            },
            seed,
            *settings,
            noise_router,
            factories.aquifer,
        )
    });
    let scan_tops = noise_tree_context_scan_tops(&mut noise_chunk, geometry);
    let mut heightmaps = NoiseTreeContextHeightmaps::new(geometry.min_y);
    let mut stats = NoiseTreeContextHeightStats::default();
    scan_noise_tree_context_heightmaps(NoiseTreeContextHeightmapScan {
        geometry,
        settings,
        noise_chunk: &mut noise_chunk,
        aquifer: aquifer.as_mut(),
        scan_top_by_column: &scan_tops.by_column,
        max_scan_top_cell_y: scan_tops.max_cell_y,
        heightmaps: &mut heightmaps,
        stats: &mut stats,
    });
    print_noise_tree_context_height_debug(pos, &heightmaps, &stats);
    heightmaps.into_tree_decoration_heights()
}

fn noise_tree_context_scan_tops(
    noise_chunk: &mut NoiseChunk,
    geometry: NoiseTreeContextGeometry,
) -> NoiseTreeContextScanTops {
    let mut by_column = [geometry.min_y; 16 * 16];
    let mut max_scan_top_y = geometry.min_y;
    for local_z in 0..16 {
        for local_x in 0..16 {
            let world_x = geometry.chunk_min_x + local_x as i32;
            let world_z = geometry.chunk_min_z + local_z as i32;
            // Java already has neighbor chunks here. This lightweight fallback
            // only needs the worldgen heightmaps, so bound the scan using the
            // same preliminary surface signal Java feeds into aquifers/surface rules.
            let top_y = (noise_chunk.preliminary_surface_level(world_x, world_z) + 64)
                .clamp(geometry.min_y, geometry.min_y + geometry.height - 1);
            by_column[local_z * 16 + local_x] = top_y;
            max_scan_top_y = max_scan_top_y.max(top_y);
        }
    }
    NoiseTreeContextScanTops {
        by_column,
        max_cell_y: (max_scan_top_y - geometry.min_y).div_euclid(geometry.cell_height),
    }
}

struct NoiseTreeContextHeightmapScan<'a> {
    geometry: NoiseTreeContextGeometry,
    settings: &'a NoiseGeneratorSettings,
    noise_chunk: &'a mut NoiseChunk,
    aquifer: Option<&'a mut NoiseBasedAquifer>,
    scan_top_by_column: &'a [i32; 16 * 16],
    max_scan_top_cell_y: i32,
    heightmaps: &'a mut NoiseTreeContextHeightmaps,
    stats: &'a mut NoiseTreeContextHeightStats,
}

fn scan_noise_tree_context_heightmaps(mut input: NoiseTreeContextHeightmapScan<'_>) {
    let geometry = input.geometry;
    'cells: for cell_x_index in 0..geometry.cell_count_xz {
        input.noise_chunk.advance_cell_x(cell_x_index);
        for cell_z_index in 0..geometry.cell_count_xz {
            for cell_y_index in (0..=input.max_scan_top_cell_y).rev() {
                input.noise_chunk.select_cell_yz(cell_y_index, cell_z_index);
                for y_in_cell in (0..geometry.cell_height).rev() {
                    let pos_y = (geometry.cell_noise_min_y + cell_y_index) * geometry.cell_height
                        + y_in_cell;
                    let factor_y = y_in_cell as f64 / geometry.cell_height as f64;
                    input.noise_chunk.update_for_y(pos_y, factor_y);
                    for x_in_cell in 0..geometry.cell_width {
                        let pos_x =
                            geometry.chunk_min_x + cell_x_index * geometry.cell_width + x_in_cell;
                        let local_x = (pos_x & 15) as usize;
                        let factor_x = x_in_cell as f64 / geometry.cell_width as f64;
                        input.noise_chunk.update_for_x(pos_x, factor_x);
                        for z_in_cell in 0..geometry.cell_width {
                            let pos_z = geometry.chunk_min_z
                                + cell_z_index * geometry.cell_width
                                + z_in_cell;
                            let local_z = (pos_z & 15) as usize;
                            let factor_z = z_in_cell as f64 / geometry.cell_width as f64;
                            input.noise_chunk.update_for_z(pos_z, factor_z);
                            let index = local_z * 16 + local_x;
                            if pos_y > input.scan_top_by_column[index] {
                                continue;
                            }
                            if input.heightmaps.is_complete_at(index) {
                                continue;
                            }
                            input.stats.density_samples += 1;
                            input.stats.lowest_sampled_y = input.stats.lowest_sampled_y.min(pos_y);
                            input.stats.highest_sampled_y =
                                input.stats.highest_sampled_y.max(pos_y);
                            let density =
                                input.noise_chunk.interpolated_density(pos_x, pos_y, pos_z);
                            let block_kind = noise_context_heightmap_block_kind(
                                input.aquifer.as_deref_mut(),
                                input.noise_chunk,
                                input.settings,
                                pos_x,
                                pos_y,
                                pos_z,
                                density,
                            );
                            if block_kind == NoiseHeightmapBlockKind::Fluid {
                                input.stats.fluid_samples += 1;
                            }
                            if block_kind == NoiseHeightmapBlockKind::Air {
                                continue;
                            }
                            if input.heightmaps.record_block(index, pos_y, block_kind) {
                                break 'cells;
                            }
                        }
                    }
                }
            }
        }
        input.noise_chunk.swap_slices();
    }
}

fn print_noise_tree_context_height_debug(
    pos: ChunkPos,
    heightmaps: &NoiseTreeContextHeightmaps,
    stats: &NoiseTreeContextHeightStats,
) {
    if std::env::var_os("VIBECRAFT_WORLDGEN_TREE_HEIGHT_DEBUG").is_some() {
        eprintln!(
            "[tree-height-debug] chunk=({}, {}) density_samples={} fluid_samples={} y_range={}..{} remaining_world_surface={} remaining_ocean_floor={}",
            pos.x,
            pos.z,
            stats.density_samples,
            stats.fluid_samples,
            stats.lowest_sampled_y,
            stats.highest_sampled_y,
            heightmaps.remaining_world_surface,
            heightmaps.remaining_ocean_floor
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NoiseHeightmapBlockKind {
    Air,
    Fluid,
    Solid,
}

fn noise_context_heightmap_block_kind(
    aquifer: Option<&mut NoiseBasedAquifer>,
    noise_chunk: &NoiseChunk,
    settings: &NoiseGeneratorSettings,
    x: i32,
    y: i32,
    z: i32,
    density: f64,
) -> NoiseHeightmapBlockKind {
    if density > 0.0 {
        return NoiseHeightmapBlockKind::Solid;
    }

    let substance = if let Some(aquifer) = aquifer {
        aquifer.compute_substance(noise_chunk, x, y, z, density)
    } else {
        Some(global_fluid_status(y, settings.sea_level, settings.default_fluid).at(y))
    };
    match substance {
        None => NoiseHeightmapBlockKind::Solid,
        Some("minecraft:water" | "minecraft:lava") => NoiseHeightmapBlockKind::Fluid,
        Some(_) => NoiseHeightmapBlockKind::Air,
    }
}
