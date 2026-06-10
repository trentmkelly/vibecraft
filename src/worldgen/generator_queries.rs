use super::*;

pub fn generator_base_height_for_stem(
    x: i32,
    z: i32,
    heightmap: HeightmapKind,
    stem: &ResolvedLevelStem,
) -> Result<i32, String> {
    match &stem.generator {
        ResolvedChunkGenerator::Flat { settings, .. } => Ok(flat_base_height(
            &settings.expanded_layers,
            FLAT_GENERATOR_MIN_Y,
            FLAT_GENERATOR_GEN_DEPTH,
            heightmap,
        )),
        ResolvedChunkGenerator::Noise { noise_settings, .. } => {
            let chunk_pos = ChunkPos {
                x: x.div_euclid(16),
                z: z.div_euclid(16),
            };
            let chunk = generator_build_surface_for_stem(chunk_pos, stem)?;
            Ok(generated_chunk_base_height(
                &chunk,
                x,
                z,
                noise_settings.noise.min_y,
                noise_settings.noise.height,
                heightmap,
            ))
        }
        ResolvedChunkGenerator::Debug { .. } => Err(format!(
            "Debug base-height query for {} is not implemented",
            stem.dimension
        )),
    }
}

pub fn generator_find_spawn_position_for_stem(
    stem: &ResolvedLevelStem,
    seed: i64,
) -> Result<BlockPos, String> {
    match &stem.generator {
        ResolvedChunkGenerator::Noise { noise_settings, .. } => {
            let router_id = noise_router_id_for_settings(**noise_settings);
            let noise_router = builtin_noise_router(router_id)
                .map(|entry| entry.router)
                .unwrap_or(NONE_NOISE_ROUTER);
            let climate_pos = climate_spawn_position(
                noise_settings.spawn_target,
                noise_router,
                **noise_settings,
                seed,
            );
            let chunk_pos = ChunkPos {
                x: climate_pos.x.div_euclid(16),
                z: climate_pos.z.div_euclid(16),
            };
            let chunk = generator_build_surface_for_stem(chunk_pos, stem)?;
            let local_x = climate_pos.x.rem_euclid(16) as usize;
            let local_z = climate_pos.z.rem_euclid(16) as usize;
            let surface_y = read_world_surface_wg(&chunk, local_x, local_z);
            let spawn_y = fixup_spawn_height(
                surface_y,
                noise_settings.noise.min_y,
                noise_settings.noise.min_y + noise_settings.noise.height,
                |y| {
                    let block = chunk
                        .get_block_state(climate_pos.x, y, climate_pos.z)
                        .unwrap_or_else(|| "minecraft:air".to_string());
                    matches!(spawn_block_kind(&block), SpawnBlockKind::Air)
                },
            );
            Ok(BlockPos {
                x: climate_pos.x,
                y: spawn_y,
                z: climate_pos.z,
            })
        }
        ResolvedChunkGenerator::Flat { settings, .. } => {
            let y = flat_base_height(
                &settings.expanded_layers,
                FLAT_GENERATOR_MIN_Y,
                FLAT_GENERATOR_GEN_DEPTH,
                HeightmapKind::MotionBlocking,
            );
            Ok(BlockPos { x: 8, y, z: 8 })
        }
        ResolvedChunkGenerator::Debug { .. } => Ok(BlockPos { x: 0, y: 80, z: 0 }),
    }
}

pub fn generator_base_column_for_stem(
    x: i32,
    z: i32,
    stem: &ResolvedLevelStem,
) -> Result<FlatNoiseColumn, String> {
    match &stem.generator {
        ResolvedChunkGenerator::Flat { settings, .. } => Ok(flat_base_column(
            &settings.expanded_layers,
            FLAT_GENERATOR_MIN_Y,
            FLAT_GENERATOR_GEN_DEPTH,
        )),
        ResolvedChunkGenerator::Noise { noise_settings, .. } => {
            let chunk_pos = ChunkPos {
                x: x.div_euclid(16),
                z: z.div_euclid(16),
            };
            let chunk = generator_build_surface_for_stem(chunk_pos, stem)?;
            Ok(generated_chunk_base_column(
                &chunk,
                x,
                z,
                noise_settings.noise.min_y,
                noise_settings.noise.height,
            ))
        }
        ResolvedChunkGenerator::Debug { .. } => Err(format!(
            "Debug base-column query for {} is not implemented",
            stem.dimension
        )),
    }
}

fn generated_chunk_base_height(
    chunk: &LevelChunk,
    x: i32,
    z: i32,
    min_y: i32,
    height: i32,
    heightmap: HeightmapKind,
) -> i32 {
    for y in (min_y..min_y + height).rev() {
        let Some(block) = chunk.get_block_state(x, y, z) else {
            continue;
        };
        if heightmap_opaque(heightmap, &block) {
            return y + 1;
        }
    }
    min_y
}

fn generated_chunk_base_column(
    chunk: &LevelChunk,
    x: i32,
    z: i32,
    min_y: i32,
    height: i32,
) -> FlatNoiseColumn {
    let states = (0..height.max(0))
        .map(|offset| {
            chunk
                .get_block_state(x, min_y + offset, z)
                .unwrap_or_else(|| "minecraft:air".to_string())
        })
        .map(|block| generated_column_static_block_name(&block).unwrap_or("minecraft:air"))
        .collect();
    FlatNoiseColumn { min_y, states }
}

fn generated_column_static_block_name(block: &str) -> Option<&'static str> {
    match block {
        "minecraft:air" => Some("minecraft:air"),
        "minecraft:bedrock" => Some("minecraft:bedrock"),
        "minecraft:stone" => Some("minecraft:stone"),
        "minecraft:granite" => Some("minecraft:granite"),
        "minecraft:diorite" => Some("minecraft:diorite"),
        "minecraft:andesite" => Some("minecraft:andesite"),
        "minecraft:deepslate" => Some("minecraft:deepslate"),
        "minecraft:dirt" => Some("minecraft:dirt"),
        "minecraft:grass_block" => Some("minecraft:grass_block"),
        "minecraft:sand" => Some("minecraft:sand"),
        "minecraft:sandstone" => Some("minecraft:sandstone"),
        "minecraft:gravel" => Some("minecraft:gravel"),
        "minecraft:water" => Some("minecraft:water"),
        "minecraft:lava" => Some("minecraft:lava"),
        _ => None,
    }
}
