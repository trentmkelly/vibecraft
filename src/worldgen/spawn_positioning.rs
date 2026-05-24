use super::*;

pub fn initial_spawn_position(
    debug_only_half_world: bool,
    debug_world_recreate: bool,
    is_debug: bool,
    spawn_chunk_x: i32,
    spawn_chunk_z: i32,
    generator_spawn_height: i32,
    min_y: i32,
    world_surface_height_at_chunk_center: i32,
) -> InitialSpawnKind {
    if debug_only_half_world && debug_world_recreate {
        InitialSpawnKind::DebugHalfWorld {
            x: 0,
            y: 64,
            z: -100,
        }
    } else if is_debug {
        InitialSpawnKind::DebugWorld { x: 0, y: 80, z: 0 }
    } else {
        let y = if generator_spawn_height < min_y {
            world_surface_height_at_chunk_center
        } else {
            generator_spawn_height
        };
        InitialSpawnKind::Normal {
            x: spawn_chunk_x * 16 + 8,
            y,
            z: spawn_chunk_z * 16 + 8,
        }
    }
}

pub fn initial_spawn_chunk_spiral_offsets() -> Vec<(i32, i32)> {
    let radius = SPAWN_SELECTION_CONSTANTS.initial_chunk_search_radius;
    let mut x_offset = 0;
    let mut z_offset = 0;
    let mut dx = 0;
    let mut dz = -1;
    let mut offsets = Vec::with_capacity(((radius * 2 + 1) * (radius * 2 + 1)) as usize);

    for _ in 0..(radius * 2 + 1).pow(2) {
        if x_offset >= -radius && x_offset <= radius && z_offset >= -radius && z_offset <= radius {
            offsets.push((x_offset, z_offset));
        }

        if x_offset == z_offset
            || (x_offset < 0 && x_offset == -z_offset)
            || (x_offset > 0 && x_offset == 1 - z_offset)
        {
            let old_dx = dx;
            dx = -dz;
            dz = old_dx;
        }

        x_offset += dx;
        z_offset += dz;
    }

    offsets
}

pub fn climate_spawn_position(
    target_climates: &[ClimateParameterPoint],
    noise_router: NoiseRouter,
    settings: NoiseGeneratorSettings,
    seed: i64,
) -> BlockPos {
    if target_climates.is_empty() {
        return BlockPos { x: 0, y: 0, z: 0 };
    }
    let sampler = ClimateSampler::from_noise_router(&noise_router, seed, settings);
    let mut best = climate_spawn_position_and_fitness(target_climates, &sampler, 0, 0);
    climate_spawn_radial_search(target_climates, &sampler, &mut best, 2048.0, 512.0);
    climate_spawn_radial_search(target_climates, &sampler, &mut best, 512.0, 32.0);
    best.location
}

fn climate_spawn_radial_search(
    target_climates: &[ClimateParameterPoint],
    sampler: &ClimateSampler,
    best: &mut ClimateSpawnCandidate,
    max_radius: f32,
    radius_increment: f32,
) {
    let mut angle = 0.0_f32;
    let mut radius = radius_increment;
    let search_origin = best.location;
    while radius <= max_radius {
        let x = search_origin.x + (angle.sin() * radius) as i32;
        let z = search_origin.z + (angle.cos() * radius) as i32;
        let candidate = climate_spawn_position_and_fitness(target_climates, sampler, x, z);
        if candidate.fitness < best.fitness {
            *best = candidate;
        }
        angle += radius_increment / radius;
        if angle > std::f32::consts::TAU {
            angle = 0.0;
            radius += radius_increment;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ClimateSpawnCandidate {
    location: BlockPos,
    fitness: i64,
}

fn climate_spawn_position_and_fitness(
    target_climates: &[ClimateParameterPoint],
    sampler: &ClimateSampler,
    block_x: i32,
    block_z: i32,
) -> ClimateSpawnCandidate {
    let target = sampler.sample(block_x.div_euclid(4), 0, block_z.div_euclid(4));
    let zero_depth_target = ClimateTarget { depth: 0, ..target };
    let min_fitness = target_climates
        .iter()
        .map(|point| point.fitness(zero_depth_target))
        .min()
        .unwrap_or(i64::MAX);
    let distance_bias =
        i64::from(block_x) * i64::from(block_x) + i64::from(block_z) * i64::from(block_z);
    ClimateSpawnCandidate {
        location: BlockPos {
            x: block_x,
            y: 0,
            z: block_z,
        },
        fitness: min_fitness
            .saturating_mul(2048_i64 * 2048_i64)
            .saturating_add(distance_bias),
    }
}

pub fn spawn_search_candidate_count(radius: i32) -> i32 {
    let side = i64::from(radius.max(0)) * 2 + 1;
    i64::from(SPAWN_SELECTION_CONSTANTS.spawn_search_absolute_max_attempts).min(side * side) as i32
}

pub fn spawn_search_coprime(candidate_count: i32) -> i32 {
    if candidate_count <= SPAWN_SELECTION_CONSTANTS.small_search_coprime_threshold {
        candidate_count - 1
    } else {
        SPAWN_SELECTION_CONSTANTS.large_search_coprime
    }
}

pub fn spawn_search_radius(respawn_radius_rule: i32, distance_to_border: i32) -> i32 {
    let mut radius = respawn_radius_rule.max(0);
    if distance_to_border < radius {
        radius = distance_to_border;
    }
    if distance_to_border <= 1 {
        radius = 1;
    }
    radius
}

pub fn spawn_search_candidate(
    spawn_x: i32,
    spawn_z: i32,
    radius: i32,
    random_offset: i32,
    candidate_index: i32,
) -> Option<(i32, i32)> {
    let candidate_count = spawn_search_candidate_count(radius);
    if candidate_index >= candidate_count {
        return None;
    }
    let side = radius.max(0) * 2 + 1;
    let value = (random_offset.rem_euclid(candidate_count)
        + spawn_search_coprime(candidate_count) * candidate_index)
        .rem_euclid(candidate_count);
    let delta_x = value % side;
    let delta_z = value / side;
    Some((spawn_x + delta_x - radius, spawn_z + delta_z - radius))
}

pub fn initial_spawn_required_chunks(center: ChunkPos) -> Vec<ChunkPos> {
    let radius = SPAWN_SELECTION_CONSTANTS.player_spawn_ticket_radius;
    let mut chunks = Vec::with_capacity(((radius * 2 + 1) * (radius * 2 + 1)) as usize);
    for z in center.z - radius..=center.z + radius {
        for x in center.x - radius..=center.x + radius {
            chunks.push(ChunkPos { x, z });
        }
    }
    chunks
}

pub fn initial_spawn_readiness_report(
    center: ChunkPos,
    generated_chunks: &[SpawnChunkStatusSnapshot],
) -> InitialSpawnReadinessReport {
    let required_status = "minecraft:full";
    let required_index = chunk_status(required_status)
        .map(|status| status.index)
        .unwrap_or(usize::MAX);
    let status_by_pos: BTreeMap<ChunkPos, &'static str> = generated_chunks
        .iter()
        .map(|snapshot| (snapshot.pos, snapshot.status))
        .collect();
    let mut missing_chunks = Vec::new();
    let mut not_ready_chunks = Vec::new();

    for pos in initial_spawn_required_chunks(center) {
        let Some(status) = status_by_pos.get(&pos).copied() else {
            missing_chunks.push(pos);
            continue;
        };
        let status_index = chunk_status(status).map(|entry| entry.index).unwrap_or(0);
        if status_index < required_index {
            not_ready_chunks.push(SpawnChunkStatusSnapshot { pos, status });
        }
    }

    InitialSpawnReadinessReport {
        center,
        required_radius: SPAWN_SELECTION_CONSTANTS.player_spawn_ticket_radius,
        required_status,
        ticket_type: "minecraft:player_spawn",
        ticket_level: crate::chunk_ticket::FULL_CHUNK_LEVEL
            - SPAWN_SELECTION_CONSTANTS.player_spawn_ticket_radius,
        missing_chunks,
        not_ready_chunks,
    }
}

pub fn initial_spawn_chunks_ready(
    center: ChunkPos,
    generated_chunks: &[SpawnChunkStatusSnapshot],
) -> bool {
    initial_spawn_readiness_report(center, generated_chunks).is_ready()
}

pub fn overworld_respawn_y(
    heights: SpawnColumnHeights,
    cave_world: bool,
    blocks_from_top_plus_one_down: &[SpawnBlockKind],
) -> Option<i32> {
    let top_y = heights.top_y;
    if top_y < heights.min_y {
        return None;
    }
    if heights.surface_y <= top_y && heights.surface_y > heights.ocean_floor_y {
        return None;
    }
    let mut y = top_y + 1;
    for block in blocks_from_top_plus_one_down {
        if y < heights.min_y {
            break;
        }
        if *block == SpawnBlockKind::Fluid {
            break;
        }
        if *block == SpawnBlockKind::Solid {
            return Some(y + 1);
        }
        y -= 1;
    }
    if cave_world && blocks_from_top_plus_one_down.is_empty() {
        None
    } else {
        None
    }
}

pub fn fixup_spawn_height(
    spawn_y: i32,
    min_y: i32,
    max_y: i32,
    no_collision_no_liquid: impl Fn(i32) -> bool,
) -> i32 {
    let mut y = spawn_y;
    while !no_collision_no_liquid(y) && y < max_y {
        y += 1;
    }
    y -= 1;
    while no_collision_no_liquid(y) && y > min_y {
        y -= 1;
    }
    y + 1
}

pub fn spawn_block_kind(block: &str) -> SpawnBlockKind {
    match block {
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" => SpawnBlockKind::Air,
        "minecraft:water" | "minecraft:lava" => SpawnBlockKind::Fluid,
        "minecraft:short_grass"
        | "minecraft:tall_grass"
        | "minecraft:snow"
        | "minecraft:fire"
        | "minecraft:soul_fire" => SpawnBlockKind::NonSolid,
        _ => SpawnBlockKind::Solid,
    }
}
