use super::*;

pub fn apply_spawn_original_mobs_to_generated_chunk(
    chunk: &mut LevelChunk,
    stem: &ResolvedLevelStem,
    world_seed: i64,
    spawn_mobs_game_rule: bool,
) -> usize {
    apply_spawn_original_mobs_to_generated_chunk_timed(
        chunk,
        stem,
        world_seed,
        spawn_mobs_game_rule,
    )
    .mobs_spawned
}

pub(super) fn apply_spawn_original_mobs_to_generated_chunk_timed(
    chunk: &mut LevelChunk,
    stem: &ResolvedLevelStem,
    world_seed: i64,
    spawn_mobs_game_rule: bool,
) -> LiveMobGenerationTimings {
    let total_started = Instant::now();
    let plan_started = Instant::now();
    let Some(plan) = spawn_original_mobs_plan_for_stem(world_seed, chunk.pos, stem) else {
        return LiveMobGenerationTimings {
            total_ms: total_started.elapsed().as_millis(),
            plan_ms: plan_started.elapsed().as_millis(),
            ..LiveMobGenerationTimings::default()
        };
    };
    let plan_ms = plan_started.elapsed().as_millis();

    let biome_started = Instant::now();
    let Some(biome) =
        spawn_original_mobs_biome_generation_settings_for_chunk(chunk, stem, plan, world_seed)
    else {
        return LiveMobGenerationTimings {
            total_ms: total_started.elapsed().as_millis(),
            plan_ms,
            biome_ms: biome_started.elapsed().as_millis(),
            ..LiveMobGenerationTimings::default()
        };
    };
    let biome_ms = biome_started.elapsed().as_millis();

    let mut random = RandomSourceKind::new(
        plan.decoration_seed,
        crate::random_source::RandomAlgorithm::Legacy,
    );

    let spawn_plan_started = Instant::now();
    let spawn_plan =
        chunk_generation_mob_spawn_plan(chunk.pos, biome, spawn_mobs_game_rule, &mut random);
    let spawn_plan_ms = spawn_plan_started.elapsed().as_millis();

    let apply_started = Instant::now();
    let mut timings = LiveMobGenerationTimings {
        plan_ms,
        biome_ms,
        spawn_plan_ms,
        batches: spawn_plan.batches.len(),
        ..LiveMobGenerationTimings::default()
    };
    for batch in spawn_plan.batches {
        apply_chunk_generation_mob_batch_to_chunk_timed(
            chunk,
            batch,
            false,
            &mut random,
            &[],
            &mut timings,
        );
    }
    timings.apply_batches_ms = apply_started.elapsed().as_millis();
    timings.total_ms = total_started.elapsed().as_millis();
    timings
}

fn spawn_original_mobs_biome_generation_settings_for_chunk(
    chunk: &LevelChunk,
    stem: &ResolvedLevelStem,
    plan: SpawnOriginalMobsPlan,
    world_seed: i64,
) -> Option<&'static BiomeGenerationSettingsModel> {
    generated_chunk_biome_generation_settings_at_block(chunk, plan.biome_sample_pos)
        .or_else(|| spawn_original_mobs_biome_generation_settings(stem, plan, world_seed))
}

fn generated_chunk_biome_generation_settings_at_block(
    chunk: &LevelChunk,
    pos: BlockPos,
) -> Option<&'static BiomeGenerationSettingsModel> {
    let quart_x = pos.x.div_euclid(4);
    let quart_y = (pos.y - 1).div_euclid(4);
    let quart_z = pos.z.div_euclid(4);
    let local_x = quart_x - chunk.pos.x * 4;
    let local_z = quart_z - chunk.pos.z * 4;
    if !(0..4).contains(&local_x) || !(0..4).contains(&local_z) {
        return None;
    }

    let min_section_y = chunk.min_section_y;
    let max_section_y = chunk
        .sections
        .iter()
        .map(|section| section.y)
        .max()
        .map(i32::from)?;
    let section_y = quart_y.div_euclid(4).clamp(min_section_y, max_section_y);
    let local_y = (quart_y - section_y * 4).clamp(0, 3);
    let section = chunk
        .sections
        .iter()
        .find(|section| i32::from(section.y) == section_y)?;
    let biomes = PalettedContainer::from_nbt(&section.biomes, BIOME_SECTION_VOLUME).ok()?;
    let index = local_y as usize * 16 + local_z as usize * 4 + local_x as usize;
    let Tag::String(biome) = biomes.get_entry(index)? else {
        return None;
    };
    biome_generation_settings(biome)
}

pub(super) fn add_timing(target: &mut u128, started: Instant) {
    *target += started.elapsed().as_millis();
}

fn spawn_original_mobs_biome_generation_settings(
    stem: &ResolvedLevelStem,
    plan: SpawnOriginalMobsPlan,
    world_seed: i64,
) -> Option<&'static BiomeGenerationSettingsModel> {
    let ResolvedChunkGenerator::Noise {
        biome_source_model,
        noise_settings,
        ..
    } = &stem.generator
    else {
        return None;
    };
    let router_id = noise_router_id_for_settings(**noise_settings);
    let router = builtin_noise_router(router_id)
        .map(|entry| entry.router)
        .unwrap_or(NONE_NOISE_ROUTER);
    let sampler = ClimateSampler::from_noise_router(&router, world_seed, **noise_settings);
    let biome = get_biome(
        biome_source_model,
        plan.biome_sample_pos.x.div_euclid(4),
        plan.biome_sample_pos.y.div_euclid(4),
        plan.biome_sample_pos.z.div_euclid(4),
        &sampler,
    )?;
    biome_generation_settings(biome)
}

pub fn spawn_original_mobs_plan_for_stem(
    world_seed: i64,
    center: ChunkPos,
    stem: &ResolvedLevelStem,
) -> Option<SpawnOriginalMobsPlan> {
    match &stem.generator {
        ResolvedChunkGenerator::Noise { noise_settings, .. } => {
            if noise_settings.disable_mob_generation {
                return None;
            }
            let min_block_x = center.x * 16;
            let min_block_z = center.z * 16;
            Some(SpawnOriginalMobsPlan {
                center,
                biome_sample_pos: BlockPos {
                    x: min_block_x,
                    y: noise_settings.noise.min_y + noise_settings.noise.height,
                    z: min_block_z,
                },
                decoration_seed: crate::random_source::decoration_seed(
                    world_seed,
                    min_block_x,
                    min_block_z,
                    RandomAlgorithm::Legacy,
                ),
            })
        }
        ResolvedChunkGenerator::Flat { .. } | ResolvedChunkGenerator::Debug { .. } => None,
    }
}

pub fn chunk_generation_mob_spawn_plan(
    chunk: ChunkPos,
    biome: &BiomeGenerationSettingsModel,
    spawn_mobs_game_rule: bool,
    random: &mut RandomSourceKind,
) -> ChunkGenerationMobSpawnPlan {
    let mobs = biome_spawns_for_category(biome, "creature");
    if mobs.is_empty() || !spawn_mobs_game_rule {
        return ChunkGenerationMobSpawnPlan {
            chunk,
            batches: Vec::new(),
        };
    }

    let min_block_x = chunk.x * 16;
    let min_block_z = chunk.z * 16;
    let mut batches = Vec::new();

    while random.next_f32() < biome.creature_spawn_probability {
        let spawner =
            mobs[select_weighted_index(mobs.iter().map(|entry| entry.weight), mobs.len(), random)];
        let count_bound = 1 + spawner.max_count - spawner.min_count;
        assert!(
            count_bound > 0,
            "chunk generation mob spawn max_count must be >= min_count"
        );
        batches.push(ChunkGenerationMobSpawnBatchPlan {
            category: "creature",
            entity_type: spawner.entity_type,
            count: spawner.min_count + random_next_i32_bound(random, count_bound),
            start_x: min_block_x + random_next_i32_bound(random, 16),
            start_z: min_block_z + random_next_i32_bound(random, 16),
        });
    }

    ChunkGenerationMobSpawnPlan { chunk, batches }
}

pub fn chunk_generation_mob_spawn_attempt_plan(
    chunk: ChunkPos,
    batch: ChunkGenerationMobSpawnBatchPlan,
    random: &mut RandomSourceKind,
) -> Vec<ChunkGenerationMobSpawnAttemptPlan> {
    let min_block_x = chunk.x * 16;
    let min_block_z = chunk.z * 16;
    let mut x = batch.start_x;
    let mut z = batch.start_z;
    let mut attempts = Vec::new();

    for mob_index in 0..batch.count {
        for attempt_index in 0..4 {
            attempts.push(ChunkGenerationMobSpawnAttemptPlan {
                mob_index,
                attempt_index,
                x,
                z,
            });

            x += random_next_i32_bound(random, 5) - random_next_i32_bound(random, 5);
            z += random_next_i32_bound(random, 5) - random_next_i32_bound(random, 5);
            while x < min_block_x
                || x >= min_block_x + 16
                || z < min_block_z
                || z >= min_block_z + 16
            {
                x = batch.start_x + random_next_i32_bound(random, 5)
                    - random_next_i32_bound(random, 5);
                z = batch.start_z + random_next_i32_bound(random, 5)
                    - random_next_i32_bound(random, 5);
            }
        }
    }

    attempts
}

pub fn chunk_generation_mob_top_non_colliding_pos(
    chunk: &LevelChunk,
    entity_type: &'static str,
    x: i32,
    z: i32,
    dimension_has_ceiling: bool,
) -> ChunkGenerationMobSpawnPositionPlan {
    let heightmap = spawn_placement_heightmap(entity_type);
    let placement_type = spawn_placement_type(entity_type);
    let local_x = x.rem_euclid(16) as usize;
    let local_z = z.rem_euclid(16) as usize;
    let mut y = chunk
        .heightmap_value(heightmap, local_x, local_z)
        .unwrap_or_else(|| chunk.compute_heightmap_values(heightmap)[local_z * 16 + local_x]);

    if dimension_has_ceiling {
        let min_y = chunk.min_section_y * 16;
        loop {
            y -= 1;
            if chunk
                .get_block_state(x, y, z)
                .as_deref()
                .is_some_and(is_surface_air)
                || y <= min_y
            {
                break;
            }
        }

        loop {
            y -= 1;
            let is_air = chunk
                .get_block_state(x, y, z)
                .as_deref()
                .is_some_and(is_surface_air);
            if !is_air || y <= min_y {
                break;
            }
        }
    }

    if placement_type == "on_ground" {
        let below_y = y - 1;
        if chunk
            .get_block_state(x, below_y, z)
            .as_deref()
            .is_some_and(spawn_pathfindable_land_block)
        {
            y = below_y;
        }
    }

    ChunkGenerationMobSpawnPositionPlan {
        entity_type,
        heightmap,
        placement_type,
        pos: BlockPos { x, y, z },
    }
}

pub fn spawn_placement_heightmap(_entity_type: &str) -> HeightmapKind {
    HeightmapKind::MotionBlockingNoLeaves
}

pub fn spawn_placement_type(entity_type: &str) -> &'static str {
    match entity_type {
        "minecraft:axolotl"
        | "minecraft:cod"
        | "minecraft:dolphin"
        | "minecraft:drowned"
        | "minecraft:elder_guardian"
        | "minecraft:glow_squid"
        | "minecraft:guardian"
        | "minecraft:nautilus"
        | "minecraft:pufferfish"
        | "minecraft:salmon"
        | "minecraft:squid"
        | "minecraft:tropical_fish" => "in_water",
        "minecraft:strider" => "in_lava",
        "minecraft:evoker"
        | "minecraft:fox"
        | "minecraft:illusioner"
        | "minecraft:panda"
        | "minecraft:phantom"
        | "minecraft:shulker"
        | "minecraft:trader_llama"
        | "minecraft:vex"
        | "minecraft:vindicator"
        | "minecraft:warden" => "no_restrictions",
        _ => "on_ground",
    }
}

pub fn chunk_generation_spawn_position_ok(
    chunk: &LevelChunk,
    entity_type: &str,
    pos: BlockPos,
) -> bool {
    match spawn_placement_type(entity_type) {
        "no_restrictions" => true,
        "in_water" => {
            chunk
                .get_block_state(pos.x, pos.y, pos.z)
                .as_deref()
                .is_some_and(|block| block == "minecraft:water")
                && !chunk
                    .get_block_state(pos.x, pos.y + 1, pos.z)
                    .as_deref()
                    .is_some_and(spawn_redstone_conductor_block)
        }
        "in_lava" => chunk
            .get_block_state(pos.x, pos.y, pos.z)
            .as_deref()
            .is_some_and(|block| block == "minecraft:lava"),
        "on_ground" => {
            chunk
                .get_block_state(pos.x, pos.y - 1, pos.z)
                .as_deref()
                .is_some_and(|block| spawn_valid_ground_block(block, entity_type))
                && chunk
                    .get_block_state(pos.x, pos.y, pos.z)
                    .as_deref()
                    .is_some_and(spawn_valid_empty_block)
                && chunk
                    .get_block_state(pos.x, pos.y + 1, pos.z)
                    .as_deref()
                    .is_some_and(spawn_valid_empty_block)
        }
        _ => false,
    }
}

pub fn chunk_generation_mob_entity_snap_plan(
    chunk: ChunkPos,
    entity_type: &'static str,
    pos: BlockPos,
    random: &mut RandomSourceKind,
) -> ChunkGenerationMobEntitySnapPlan {
    let width = entity_type_width(entity_type);
    let min_x = f64::from(chunk.x * 16) + f64::from(width);
    let max_x = f64::from(chunk.x * 16) + 16.0 - f64::from(width);
    let min_z = f64::from(chunk.z * 16) + f64::from(width);
    let max_z = f64::from(chunk.z * 16) + 16.0 - f64::from(width);
    ChunkGenerationMobEntitySnapPlan {
        entity_type,
        width,
        x: f64::from(pos.x).clamp(min_x, max_x),
        y: f64::from(pos.y),
        z: f64::from(pos.z).clamp(min_z, max_z),
        yaw: random.next_f32() * 360.0,
        pitch: 0.0,
    }
}

pub fn entity_type_width(entity_type: &str) -> f32 {
    match entity_type {
        "minecraft:chicken" => 0.4,
        "minecraft:cow" | "minecraft:mooshroom" | "minecraft:pig" | "minecraft:sheep" => 0.9,
        "minecraft:donkey" | "minecraft:horse" | "minecraft:mule" => 1.3964844,
        "minecraft:rabbit" => 0.49,
        "minecraft:polar_bear" => 1.4,
        _ => 0.6,
    }
}

pub fn chunk_generation_mob_collision_plan(
    snap: ChunkGenerationMobEntitySnapPlan,
) -> ChunkGenerationMobCollisionPlan {
    let width = snap.width;
    let height = entity_type_height(snap.entity_type);
    let half_width = f64::from(width) / 2.0;
    ChunkGenerationMobCollisionPlan {
        entity_type: snap.entity_type,
        width,
        height,
        min_x: snap.x - half_width,
        min_y: snap.y,
        min_z: snap.z - half_width,
        max_x: snap.x + half_width,
        max_y: snap.y + f64::from(height),
        max_z: snap.z + half_width,
    }
}

pub fn chunk_generation_mob_no_collision(
    chunk: &LevelChunk,
    collision: ChunkGenerationMobCollisionPlan,
) -> bool {
    let min_x = collision.min_x.floor() as i32;
    let min_y = collision.min_y.floor() as i32;
    let min_z = collision.min_z.floor() as i32;
    let max_x = (collision.max_x - 1.0E-7).floor() as i32;
    let max_y = (collision.max_y - 1.0E-7).floor() as i32;
    let max_z = (collision.max_z - 1.0E-7).floor() as i32;

    for y in min_y..=max_y {
        for z in min_z..=max_z {
            for x in min_x..=max_x {
                if chunk
                    .get_block_state(x, y, z)
                    .as_deref()
                    .is_some_and(spawn_colliding_block)
                {
                    return false;
                }
            }
        }
    }

    true
}

pub fn chunk_generation_mob_spawn_rules_ok(
    chunk: &LevelChunk,
    entity_type: &str,
    pos: BlockPos,
) -> bool {
    match entity_type {
        "minecraft:mooshroom" => {
            chunk_generation_is_bright_enough_to_spawn(chunk, pos)
                && chunk
                    .get_block_state(pos.x, pos.y - 1, pos.z)
                    .as_deref()
                    .is_some_and(|block| block == "minecraft:mycelium")
        }
        "minecraft:goat" => {
            chunk_generation_is_bright_enough_to_spawn(chunk, pos)
                && chunk
                    .get_block_state(pos.x, pos.y - 1, pos.z)
                    .as_deref()
                    .is_some_and(goats_spawnable_on)
        }
        "minecraft:rabbit" => {
            chunk_generation_is_bright_enough_to_spawn(chunk, pos)
                && chunk
                    .get_block_state(pos.x, pos.y - 1, pos.z)
                    .as_deref()
                    .is_some_and(rabbits_spawnable_on)
        }
        "minecraft:chicken"
        | "minecraft:cow"
        | "minecraft:donkey"
        | "minecraft:happy_ghast"
        | "minecraft:horse"
        | "minecraft:llama"
        | "minecraft:mule"
        | "minecraft:panda"
        | "minecraft:pig"
        | "minecraft:polar_bear"
        | "minecraft:sheep"
        | "minecraft:trader_llama" => {
            chunk_generation_is_bright_enough_to_spawn(chunk, pos)
                && chunk
                    .get_block_state(pos.x, pos.y - 1, pos.z)
                    .as_deref()
                    .is_some_and(animals_spawnable_on)
        }
        _ => true,
    }
}

pub fn chunk_generation_is_bright_enough_to_spawn(chunk: &LevelChunk, pos: BlockPos) -> bool {
    chunk_generation_raw_brightness(chunk, pos, 0) > 8
}

pub fn chunk_generation_raw_brightness(chunk: &LevelChunk, pos: BlockPos, sky_dampen: i32) -> i32 {
    if chunk_generation_column_has_sky(chunk, pos) {
        (15 - sky_dampen).max(0)
    } else {
        0
    }
}

fn chunk_generation_column_has_sky(chunk: &LevelChunk, pos: BlockPos) -> bool {
    let top_y = chunk
        .sections
        .iter()
        .map(|section| i32::from(section.y) * 16 + 15)
        .max()
        .unwrap_or(pos.y);
    ((pos.y + 1)..=top_y).all(|y| {
        !chunk
            .get_block_state(pos.x, y, pos.z)
            .as_deref()
            .is_some_and(spawn_colliding_block)
    })
}

pub fn entity_type_height(entity_type: &str) -> f32 {
    match entity_type {
        "minecraft:chicken" => 0.7,
        "minecraft:cow" | "minecraft:mooshroom" => 1.4,
        "minecraft:pig" | "minecraft:sheep" => 0.9,
        "minecraft:donkey" | "minecraft:horse" | "minecraft:mule" => 1.6,
        "minecraft:rabbit" => 0.5,
        "minecraft:polar_bear" => 1.4,
        _ => 1.8,
    }
}
