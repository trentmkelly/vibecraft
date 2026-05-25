use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveChunkGenerationMode {
    Preview,
    RealSurface,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LiveChunkGenerationTimings {
    pub resolve_preset_ms: u128,
    pub terrain_ms: u128,
    pub base_generation_ms: u128,
    pub region_biome_steps_ms: u128,
    pub carvers_ms: u128,
    pub carver_blocks: usize,
    pub underground_structures_ms: u128,
    pub underground_structure_blocks: usize,
    pub ore_decoration_ms: u128,
    pub ore_blocks: usize,
    pub tree_context_ms: u128,
    pub tree_context_chunks: usize,
    pub tree_decoration_ms: u128,
    pub tree_blocks: usize,
    pub terrain: LiveTerrainTimings,
    pub heightmaps: LiveHeightmapTimings,
    pub mobs: LiveMobGenerationTimings,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LiveTerrainTimings {
    pub fill_total_ms: u128,
    pub fill_init_sections_ms: u128,
    pub fill_noise_chunk_init_ms: u128,
    pub fill_aquifer_init_ms: u128,
    pub fill_block_loop_ms: u128,
    pub fill_density_lookup_us: u128,
    pub fill_aquifer_compute_us: u128,
    pub fill_ore_vein_lookup_us: u128,
    pub fill_ore_decision_us: u128,
    pub fill_interpolation_update_us: u128,
    pub fill_full_noise_cache_ms: u128,
    pub fill_full_noise_cache_us: u128,
    pub fill_vein_noise_cache_ms: u128,
    pub fill_vein_noise_cache_us: u128,
    pub fill_heightmap_pack_ms: u128,
    pub biome_storage_ms: u128,
    pub surface_total_ms: u128,
    pub surface_noise_setup_ms: u128,
    pub surface_prelim_ms: u128,
    pub surface_column_loop_ms: u128,
    pub cell_columns: usize,
    pub full_noise_cache_fills: usize,
    pub vein_noise_cache_fills: usize,
    pub cache_once_scalar_hits: usize,
    pub cache_once_scalar_misses: usize,
    pub cache_once_array_hits: usize,
    pub cache_once_array_misses: usize,
    pub block_samples: usize,
    pub block_writes: usize,
    pub aquifer_calls: usize,
    pub ore_vein_samples: usize,
    pub interpolator_count: usize,
    pub surface_columns: usize,
    pub surface_block_samples: usize,
    pub surface_block_writes: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LiveHeightmapTimings {
    pub total_ms: u128,
    pub decode_sections_ms: u128,
    pub scan_blocks_ms: u128,
    pub pack_store_ms: u128,
    pub sections_decoded: usize,
    pub block_samples: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LiveMobGenerationTimings {
    pub total_ms: u128,
    pub plan_ms: u128,
    pub biome_ms: u128,
    pub spawn_plan_ms: u128,
    pub apply_batches_ms: u128,
    pub top_position_ms: u128,
    pub position_ok_ms: u128,
    pub snap_collision_ms: u128,
    pub spawn_rules_ms: u128,
    pub queue_ms: u128,
    pub random_walk_ms: u128,
    pub batches: usize,
    pub attempts: usize,
    pub mobs_spawned: usize,
}

pub(super) struct LiveNoiseGenerationContext {
    pub(super) noise_chunk: NoiseChunk,
    pub(super) aquifer: Option<NoiseBasedAquifer>,
}

pub fn generate_chunk_for_stem(
    pos: ChunkPos,
    stem: &ResolvedLevelStem,
) -> Result<LevelChunk, String> {
    generate_chunk_for_stem_with_mode(pos, stem, LiveChunkGenerationMode::Preview, 0)
}

pub fn generate_chunk_for_stem_with_mode(
    pos: ChunkPos,
    stem: &ResolvedLevelStem,
    mode: LiveChunkGenerationMode,
    seed: i64,
) -> Result<LevelChunk, String> {
    let mut chunk = match &stem.generator {
        ResolvedChunkGenerator::Flat { settings, .. } => materialize_flat_chunk(pos, settings),
        ResolvedChunkGenerator::Noise {
            biome_source_model,
            noise_settings,
            ..
        } => match mode {
            LiveChunkGenerationMode::Preview => {
                materialize_noise_preview_chunk(pos, biome_source_model, noise_settings)
            }
            LiveChunkGenerationMode::RealSurface => {
                match generate_real_surface_base_chunk(
                    pos,
                    biome_source_model,
                    noise_settings,
                    seed,
                ) {
                    Some((mut chunk, _terrain_timings, mut noise_context)) => {
                        apply_configured_carvers_for_biome_source_with_noise_context(
                            &mut chunk,
                            biome_source_model,
                            noise_settings,
                            seed,
                            &mut noise_context,
                        );
                        apply_mineshaft_underground_structures_to_chunk(&mut chunk, seed);
                        apply_underground_ore_decoration_to_chunk(
                            &mut chunk,
                            biome_source_model,
                            noise_settings,
                            seed,
                            None,
                        );
                        apply_initial_tree_decoration_to_chunk(
                            &mut chunk,
                            biome_source_model,
                            noise_settings,
                            seed,
                            None,
                            None,
                            None,
                        );
                        chunk
                    }
                    None => {
                        let router_id = noise_router_id_for_settings(**noise_settings);
                        let noise_router = builtin_noise_router(router_id)
                            .map(|e| e.router)
                            .unwrap_or(NONE_NOISE_ROUTER);
                        let (mut chunk, _terrain_timings) =
                            fill_from_noise_chunk_timed(pos, noise_settings, seed, noise_router);
                        chunk.status = "minecraft:surface".to_string();
                        chunk
                    }
                }
            }
        },
        ResolvedChunkGenerator::Debug { .. } => {
            return Err(format!(
                "Debug chunk generation for {} is not implemented",
                stem.dimension
            ))
        }
    };
    add_client_heightmaps_from_blocks(&mut chunk);
    Ok(chunk)
}

pub(super) fn add_client_heightmaps_from_blocks(chunk: &mut LevelChunk) {
    let _ = add_client_heightmaps_from_blocks_timed(chunk);
}

pub(super) fn add_client_heightmaps_from_blocks_timed(
    chunk: &mut LevelChunk,
) -> LiveHeightmapTimings {
    let total_started = Instant::now();
    if final_client_heightmaps_present(chunk) {
        return LiveHeightmapTimings {
            total_ms: total_started.elapsed().as_millis(),
            ..LiveHeightmapTimings::default()
        };
    }
    let decode_started = Instant::now();
    let mut sections: Vec<_> = chunk
        .sections
        .iter()
        .filter_map(|section| {
            PalettedContainer::from_nbt(&section.block_states, SECTION_VOLUME)
                .ok()
                .map(|container| (section.y, container))
        })
        .collect();
    sections.sort_by_key(|(section_y, _)| *section_y);
    let decode_sections_ms = decode_started.elapsed().as_millis();

    let scan_started = Instant::now();
    let mut heightmaps = ClientHeightmapAccumulator::new();
    let block_samples = scan_client_heightmaps_from_sections(sections, &mut heightmaps);
    let scan_blocks_ms = scan_started.elapsed().as_millis();

    let pack_started = Instant::now();
    store_final_client_heightmaps(chunk, heightmaps);
    let pack_store_ms = pack_started.elapsed().as_millis();

    LiveHeightmapTimings {
        total_ms: total_started.elapsed().as_millis(),
        decode_sections_ms,
        scan_blocks_ms,
        pack_store_ms,
        sections_decoded: chunk.sections.len(),
        block_samples,
    }
}

struct ClientHeightmapAccumulator {
    world_surface: [i32; 16 * 16],
    ocean_floor: [i32; 16 * 16],
    motion_blocking: [i32; 16 * 16],
    motion_blocking_no_leaves: [i32; 16 * 16],
    found_world_surface: [bool; 16 * 16],
    found_ocean_floor: [bool; 16 * 16],
    found_motion_blocking: [bool; 16 * 16],
    found_motion_blocking_no_leaves: [bool; 16 * 16],
    remaining_world_surface: usize,
    remaining_ocean_floor: usize,
    remaining_motion_blocking: usize,
    remaining_motion_blocking_no_leaves: usize,
}

impl ClientHeightmapAccumulator {
    fn new() -> Self {
        Self {
            world_surface: [0; 16 * 16],
            ocean_floor: [0; 16 * 16],
            motion_blocking: [0; 16 * 16],
            motion_blocking_no_leaves: [0; 16 * 16],
            found_world_surface: [false; 16 * 16],
            found_ocean_floor: [false; 16 * 16],
            found_motion_blocking: [false; 16 * 16],
            found_motion_blocking_no_leaves: [false; 16 * 16],
            remaining_world_surface: 16 * 16,
            remaining_ocean_floor: 16 * 16,
            remaining_motion_blocking: 16 * 16,
            remaining_motion_blocking_no_leaves: 16 * 16,
        }
    }

    fn complete_at(&self, column: usize) -> bool {
        self.found_world_surface[column]
            && self.found_ocean_floor[column]
            && self.found_motion_blocking[column]
            && self.found_motion_blocking_no_leaves[column]
    }

    fn complete_all(&self) -> bool {
        self.remaining_world_surface == 0
            && self.remaining_ocean_floor == 0
            && self.remaining_motion_blocking == 0
            && self.remaining_motion_blocking_no_leaves == 0
    }

    fn record_block(&mut self, column: usize, world_height: i32, block: &str) {
        if !self.found_world_surface[column] && heightmap_opaque(HeightmapKind::WorldSurface, block)
        {
            self.world_surface[column] = world_height;
            self.found_world_surface[column] = true;
            self.remaining_world_surface -= 1;
        }
        if !self.found_ocean_floor[column] && heightmap_opaque(HeightmapKind::OceanFloor, block) {
            self.ocean_floor[column] = world_height;
            self.found_ocean_floor[column] = true;
            self.remaining_ocean_floor -= 1;
        }
        if !self.found_motion_blocking[column]
            && heightmap_opaque(HeightmapKind::MotionBlocking, block)
        {
            self.motion_blocking[column] = world_height;
            self.found_motion_blocking[column] = true;
            self.remaining_motion_blocking -= 1;
        }
        if !self.found_motion_blocking_no_leaves[column]
            && heightmap_opaque(HeightmapKind::MotionBlockingNoLeaves, block)
        {
            self.motion_blocking_no_leaves[column] = world_height;
            self.found_motion_blocking_no_leaves[column] = true;
            self.remaining_motion_blocking_no_leaves -= 1;
        }
    }
}

fn scan_client_heightmaps_from_sections(
    sections: Vec<(i8, PalettedContainer)>,
    heightmaps: &mut ClientHeightmapAccumulator,
) -> usize {
    let mut block_samples = 0;
    'sections: for (section_y, container) in sections.into_iter().rev() {
        let section_min_y = i32::from(section_y) * 16;
        for local_y in (0..16).rev() {
            let world_height = section_min_y + local_y as i32 + 1;
            for z in 0..16 {
                for x in 0..16 {
                    let column = z * 16 + x;
                    if heightmaps.complete_at(column) {
                        continue;
                    }
                    let index = local_y * 256 + z * 16 + x;
                    let Some(block) = container.get_entry(index).and_then(block_name_from_tag)
                    else {
                        continue;
                    };
                    block_samples += 1;
                    heightmaps.record_block(column, world_height, block);
                    if heightmaps.complete_all() {
                        break 'sections;
                    }
                }
            }
        }
    }
    block_samples
}

fn store_final_client_heightmaps(chunk: &mut LevelChunk, heightmaps: ClientHeightmapAccumulator) {
    for (name, values) in [
        ("WORLD_SURFACE", heightmaps.world_surface),
        ("OCEAN_FLOOR", heightmaps.ocean_floor),
        ("MOTION_BLOCKING", heightmaps.motion_blocking),
        (
            "MOTION_BLOCKING_NO_LEAVES",
            heightmaps.motion_blocking_no_leaves,
        ),
    ] {
        chunk
            .heightmaps
            .entry(name.to_string())
            .or_insert_with(|| Tag::LongArray(pack_heightmap(values)));
    }
}

fn final_client_heightmaps_present(chunk: &LevelChunk) -> bool {
    [
        HeightmapKind::WorldSurface,
        HeightmapKind::OceanFloor,
        HeightmapKind::MotionBlocking,
        HeightmapKind::MotionBlockingNoLeaves,
    ]
    .iter()
    .all(|heightmap| chunk.heightmaps.contains_key(heightmap.storage_name()))
}

pub(super) fn add_client_heightmaps_from_generated_sections(
    chunk: &mut LevelChunk,
    section_blocks: &GeneratedSectionBlocks,
) {
    let mut world_surface = [0_i32; 16 * 16];
    let mut ocean_floor = [0_i32; 16 * 16];
    let mut motion_blocking = [0_i32; 16 * 16];
    let mut motion_blocking_no_leaves = [0_i32; 16 * 16];
    let mut found_world_surface = [false; 16 * 16];
    let mut found_ocean_floor = [false; 16 * 16];
    let mut found_motion_blocking = [false; 16 * 16];
    let mut found_motion_blocking_no_leaves = [false; 16 * 16];
    let mut remaining_world_surface = 16 * 16;
    let mut remaining_ocean_floor = 16 * 16;
    let mut remaining_motion_blocking = 16 * 16;
    let mut remaining_motion_blocking_no_leaves = 16 * 16;

    'sections: for (section_index, section) in section_blocks.sections.iter().enumerate().rev() {
        if section.non_air_blocks == 0 {
            continue;
        }
        let section_min_y = (section_blocks.min_section_y + section_index as i32) * 16;
        for local_y in (0..16).rev() {
            let world_height = section_min_y + local_y as i32 + 1;
            for z in 0..16 {
                for x in 0..16 {
                    let column = z * 16 + x;
                    if found_world_surface[column]
                        && found_ocean_floor[column]
                        && found_motion_blocking[column]
                        && found_motion_blocking_no_leaves[column]
                    {
                        continue;
                    }
                    let index = local_y * 256 + z * 16 + x;
                    let block = section_blocks
                        .palette_names
                        .get(section.ids[index] as usize)
                        .map(String::as_str)
                        .unwrap_or("minecraft:air");
                    if block == "minecraft:air" {
                        continue;
                    }
                    if !found_world_surface[column]
                        && heightmap_opaque(HeightmapKind::WorldSurface, block)
                    {
                        world_surface[column] = world_height;
                        found_world_surface[column] = true;
                        remaining_world_surface -= 1;
                    }
                    if !found_ocean_floor[column]
                        && heightmap_opaque(HeightmapKind::OceanFloor, block)
                    {
                        ocean_floor[column] = world_height;
                        found_ocean_floor[column] = true;
                        remaining_ocean_floor -= 1;
                    }
                    if !found_motion_blocking[column]
                        && heightmap_opaque(HeightmapKind::MotionBlocking, block)
                    {
                        motion_blocking[column] = world_height;
                        found_motion_blocking[column] = true;
                        remaining_motion_blocking -= 1;
                    }
                    if !found_motion_blocking_no_leaves[column]
                        && heightmap_opaque(HeightmapKind::MotionBlockingNoLeaves, block)
                    {
                        motion_blocking_no_leaves[column] = world_height;
                        found_motion_blocking_no_leaves[column] = true;
                        remaining_motion_blocking_no_leaves -= 1;
                    }
                    if remaining_world_surface == 0
                        && remaining_ocean_floor == 0
                        && remaining_motion_blocking == 0
                        && remaining_motion_blocking_no_leaves == 0
                    {
                        break 'sections;
                    }
                }
            }
        }
    }

    for (heightmap, values) in [
        (HeightmapKind::WorldSurface, world_surface),
        (HeightmapKind::OceanFloor, ocean_floor),
        (HeightmapKind::MotionBlocking, motion_blocking),
        (
            HeightmapKind::MotionBlockingNoLeaves,
            motion_blocking_no_leaves,
        ),
    ] {
        chunk
            .heightmaps
            .entry(heightmap.storage_name().to_string())
            .or_insert_with(|| Tag::LongArray(pack_heightmap(values)));
    }
}

fn block_name_from_tag(tag: &Tag) -> Option<&str> {
    match tag {
        Tag::Compound(fields) => {
            fields
                .iter()
                .find(|(name, _)| name == "Name")
                .and_then(|(_, value)| match value {
                    Tag::String(name) => Some(name.as_str()),
                    _ => None,
                })
        }
        Tag::String(name) => Some(name.as_str()),
        _ => None,
    }
}

fn generated_chunk_with_status(
    pos: ChunkPos,
    stem: &ResolvedLevelStem,
    status: &'static str,
) -> Result<LevelChunk, String> {
    let mut chunk = generate_chunk_for_stem(pos, stem)?;
    chunk.status = status.to_string();
    Ok(chunk)
}

pub fn generator_create_structures_for_stem(
    pos: ChunkPos,
    stem: &ResolvedLevelStem,
) -> Result<LevelChunk, String> {
    generated_chunk_with_status(pos, stem, "minecraft:structure_starts")
}

pub fn generator_create_references_for_stem(
    pos: ChunkPos,
    stem: &ResolvedLevelStem,
) -> Result<LevelChunk, String> {
    generated_chunk_with_status(pos, stem, "minecraft:structure_references")
}

pub fn generator_create_biomes_for_stem(
    pos: ChunkPos,
    stem: &ResolvedLevelStem,
) -> Result<LevelChunk, String> {
    generated_chunk_with_status(pos, stem, "minecraft:biomes")
}

pub fn generator_fill_from_noise_for_stem(
    pos: ChunkPos,
    stem: &ResolvedLevelStem,
) -> Result<LevelChunk, String> {
    generated_chunk_with_status(pos, stem, "minecraft:noise")
}

pub fn generator_build_surface_for_stem(
    pos: ChunkPos,
    stem: &ResolvedLevelStem,
) -> Result<LevelChunk, String> {
    match &stem.generator {
        ResolvedChunkGenerator::Noise {
            biome_source_model,
            noise_settings,
            ..
        } => {
            let router_id = noise_router_id_for_settings(**noise_settings);
            let noise_router = builtin_noise_router(router_id)
                .map(|e| e.router)
                .unwrap_or(NONE_NOISE_ROUTER);
            match load_surface_rule(noise_settings.id) {
                Some(rule) => Ok(fill_noise_and_build_surface(
                    pos,
                    biome_source_model,
                    noise_settings,
                    0,
                    noise_router,
                    &rule,
                )),
                None => {
                    // No surface rule found — fall back to noise-only chunk.
                    let mut chunk = fill_from_noise_chunk(pos, noise_settings, 0, noise_router);
                    chunk.status = "minecraft:surface".to_string();
                    Ok(chunk)
                }
            }
        }
        _ => {
            // Flat / Debug generators: surface is identical to noise phase.
            let mut chunk = generate_chunk_for_stem(pos, stem)?;
            chunk.status = "minecraft:surface".to_string();
            Ok(chunk)
        }
    }
}

pub fn generator_apply_carvers_for_stem(
    pos: ChunkPos,
    stem: &ResolvedLevelStem,
) -> Result<LevelChunk, String> {
    match &stem.generator {
        ResolvedChunkGenerator::Noise {
            biome_source_model,
            noise_settings,
            ..
        } => {
            let mut chunk = generator_build_surface_for_stem(pos, stem)?;
            let carvers = carvers_for_biome_source_and_noise_settings(
                biome_source_model,
                pos,
                noise_settings,
                0,
            );
            apply_configured_carvers_to_chunk(&mut chunk, noise_settings, 0, carvers);
            add_client_heightmaps_from_blocks(&mut chunk);
            chunk.status = "minecraft:carvers".to_string();
            Ok(chunk)
        }
        _ => generated_chunk_with_status(pos, stem, "minecraft:carvers"),
    }
}

pub fn generator_apply_biome_decoration_for_stem(
    pos: ChunkPos,
    stem: &ResolvedLevelStem,
) -> Result<LevelChunk, String> {
    generated_chunk_with_status(pos, stem, "minecraft:features")
}

pub fn generator_spawn_original_mobs_for_stem(
    pos: ChunkPos,
    stem: &ResolvedLevelStem,
) -> Result<LevelChunk, String> {
    generator_spawn_original_mobs_for_stem_with_seed(pos, stem, 0, true)
}

pub fn generator_spawn_original_mobs_for_stem_with_seed(
    pos: ChunkPos,
    stem: &ResolvedLevelStem,
    world_seed: i64,
    spawn_mobs_game_rule: bool,
) -> Result<LevelChunk, String> {
    let mut chunk = generator_apply_biome_decoration_for_stem(pos, stem)?;
    apply_spawn_original_mobs_to_generated_chunk(
        &mut chunk,
        stem,
        world_seed,
        spawn_mobs_game_rule,
    );
    chunk.status = "minecraft:spawn".to_string();
    Ok(chunk)
}
