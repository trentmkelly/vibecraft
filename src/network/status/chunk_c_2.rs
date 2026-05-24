use super::*;

pub fn write_play_chunk_delta(
    stream: &mut TcpStream,
    compression: CompressionState,
    center_chunk_x: i32,
    center_chunk_z: i32,
    chunks: &[(i32, i32)],
    update_cache_center: bool,
    world_root: &Path,
    world_seed: i64,
    chunk_cache: &GeneratedChunkCache,
    mut live_fluid_ticks: Option<(&mut LiveFluidTicks, i64, &WorldLayout)>,
) -> io::Result<()> {
    let batch_started = Instant::now();
    eprintln!(
        "[chunk-batch-timing] start center=({}, {}) chunks={} update_center={} live_fluid_seed={}",
        center_chunk_x,
        center_chunk_z,
        chunks.len(),
        update_cache_center,
        live_fluid_ticks.is_some()
    );
    if update_cache_center {
        write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID,
            |payload| {
                write_var_i32(payload, center_chunk_x)?;
                write_var_i32(payload, center_chunk_z)
            },
        )?;
    }
    if chunks.is_empty() {
        eprintln!(
            "[chunk-batch-timing] finish center=({}, {}) chunks=0 elapsed={}ms",
            center_chunk_x,
            center_chunk_z,
            batch_started.elapsed().as_millis()
        );
        return Ok(());
    }
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_PLAY_CHUNK_BATCH_START_PACKET_ID,
        |_payload| Ok(()),
    )?;

    // Java schedules chunk status work on the worldgen background executor
    // (`NoiseBasedChunkGenerator.fillFromNoise` uses `supplyAsync(...,
    // Util.backgroundExecutor().forName("wgen_fill_noise"))`) and lets the
    // client receive ready chunks progressively. Generate the complete
    // configured view-distance set, but do not wait for the entire square before
    // sending the first finished chunks.
    let workers = thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(4)
        .clamp(1, 4);
    let worker_count = chunks.len().min(workers);
    let queue = Arc::new(Mutex::new(VecDeque::from(chunks.to_vec())));
    let (sender, receiver) = mpsc::channel::<(i32, i32, Arc<LevelChunk>)>();
    thread::scope(|scope| {
        for _ in 0..worker_count {
            let queue = Arc::clone(&queue);
            let sender = sender.clone();
            let cache = chunk_cache.clone();
            scope.spawn(move || loop {
                let Some((x, z)) = queue.lock().unwrap().pop_front() else {
                    break;
                };
                let chunk = cache.get_or_load(x, z, world_root, world_seed);
                if sender.send((x, z, chunk)).is_err() {
                    break;
                }
            });
        }
        drop(sender);

        for received in 0..chunks.len() {
            let recv_started = Instant::now();
            let (_x, _z, chunk) = receiver.recv().map_err(|err| {
                io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    format!("chunk generation worker stopped before batch completed: {err}"),
                )
            })?;
            let recv_ms = recv_started.elapsed().as_millis();
            let write_started = Instant::now();
            if let Some((ticks, game_time, _layout)) = live_fluid_ticks.as_mut() {
                unpack_chunk_fluid_ticks(&mut **ticks, *game_time, &chunk);
            }
            write_generated_spawn_chunk_packets_from_chunk(stream, compression, &chunk)?;
            let write_ms = write_started.elapsed().as_millis();
            if write_ms >= 10 || recv_ms >= 10 || received + 1 == chunks.len() {
                eprintln!(
                    "[chunk-batch-timing] progress center=({}, {}) sent={}/{} chunk=({}, {}) recv_wait={}ms write={}ms elapsed={}ms",
                    center_chunk_x,
                    center_chunk_z,
                    received + 1,
                    chunks.len(),
                    chunk.pos.x,
                    chunk.pos.z,
                    recv_ms,
                    write_ms,
                    batch_started.elapsed().as_millis()
                );
            }
        }
        Ok::<(), io::Error>(())
    })?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_PLAY_CHUNK_BATCH_FINISHED_PACKET_ID,
        |payload| write_var_i32(payload, chunks.len() as i32),
    )?;
    eprintln!(
        "[chunk-batch-timing] finish center=({}, {}) chunks={} elapsed={}ms",
        center_chunk_x,
        center_chunk_z,
        chunks.len(),
        batch_started.elapsed().as_millis()
    );
    Ok(())
}

pub fn write_generated_spawn_chunk_packets_from_chunk<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    chunk: &LevelChunk,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_PLAY_LEVEL_CHUNK_WITH_LIGHT_PACKET_ID,
        |payload| write_generated_spawn_chunk_payload(payload, chunk),
    )?;
    for plan in generated_chunk_entity_spawn_plans(chunk) {
        write_generated_chunk_entity_spawn_packets(writer, compression, &plan)?;
    }
    Ok(())
}

pub fn chunk_batch_radius(properties: &ServerProperties) -> i32 {
    (properties.view_distance as i32).clamp(MIN_CHUNK_BATCH_RADIUS, MAX_CHUNK_BATCH_RADIUS)
}

pub fn chunk_batch_size(radius: i32) -> i32 {
    (radius * 2 + 1) * (radius * 2 + 1)
}

pub fn delay_initial_chunk_batch_for_probe(
    stream: &mut TcpStream,
    compression: CompressionState,
) -> io::Result<()> {
    let delay_ms = env::var("RUSTCRAFT_INITIAL_CHUNK_DELAY_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    if delay_ms == 0 {
        return Ok(());
    }

    let deadline = Instant::now() + Duration::from_millis(delay_ms);
    let mut last_keep_alive = Instant::now();
    let mut keep_alive_id = 0_i64;
    loop {
        let now = Instant::now();
        if now >= deadline {
            break;
        }
        if now.duration_since(last_keep_alive) >= PLAY_KEEP_ALIVE_INTERVAL {
            keep_alive_id = keep_alive_id.wrapping_add(1);
            write_framed_packet_with_compression(
                stream,
                compression,
                CLIENTBOUND_KEEP_ALIVE_PACKET_ID,
                |payload| payload.write_all(&keep_alive_id.to_be_bytes()),
            )?;
            last_keep_alive = now;
        }
        thread::sleep(Duration::from_millis(25).min(deadline.saturating_duration_since(now)));
    }
    Ok(())
}

pub fn chunk_window(center_chunk_x: i32, center_chunk_z: i32, radius: i32) -> BTreeSet<(i32, i32)> {
    let mut chunks = BTreeSet::new();
    for z in (center_chunk_z - radius)..=(center_chunk_z + radius) {
        for x in (center_chunk_x - radius)..=(center_chunk_x + radius) {
            chunks.insert((x, z));
        }
    }
    chunks
}

pub fn newly_visible_chunks(
    previous: &BTreeSet<(i32, i32)>,
    next: &BTreeSet<(i32, i32)>,
) -> Vec<(i32, i32)> {
    next.difference(previous).copied().collect()
}

pub fn write_forget_level_chunk_packet(
    stream: &mut impl Write,
    compression: CompressionState,
    x: i32,
    z: i32,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID,
        |payload| payload.write_all(&packed_chunk_pos(x, z).to_be_bytes()),
    )
}

pub fn write_forget_generated_spawn_chunk_packets<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    x: i32,
    z: i32,
    world_root: &Path,
    world_seed: i64,
    chunk_cache: &GeneratedChunkCache,
) -> io::Result<()> {
    let chunk = chunk_cache.get_or_load(x, z, world_root, world_seed);
    write_generated_chunk_entity_remove_packets(writer, compression, &chunk)?;
    write_forget_level_chunk_packet(writer, compression, x, z)
}

pub fn write_generated_chunk_entity_remove_packets<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    chunk: &LevelChunk,
) -> io::Result<()> {
    let entity_ids: Vec<i32> = generated_chunk_entity_add_packets(chunk)
        .into_iter()
        .map(|packet| packet.id)
        .collect();
    if entity_ids.is_empty() {
        return Ok(());
    }
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID,
        |payload| ClientboundRemoveEntitiesPacket { entity_ids }.write(payload),
    )
}

pub fn packed_chunk_pos(x: i32, z: i32) -> i64 {
    (i64::from(x) & 0xffff_ffff) | ((i64::from(z) & 0xffff_ffff) << 32)
}

pub fn chunk_coordinate(block_coordinate: f64) -> i32 {
    (block_coordinate.floor() as i32).div_euclid(16)
}

pub fn write_player_info_initializing_packet<W: Write>(
    writer: &mut W,
    profile: &NameAndId,
    game_mode: GameMode,
) -> io::Result<()> {
    writer.write_all(&[0xff])?;
    write_var_i32(writer, 1)?;
    write_uuid(writer, uuid_from_hyphenated(&profile.uuid)?)?;
    crate::network::codec::write_string(writer, &profile.name, 16)?;
    write_var_i32(writer, 0)?;
    write_bool(writer, false)?;
    write_var_i32(writer, game_mode_legacy_id(game_mode))?;
    write_bool(writer, true)?;
    write_var_i32(writer, 0)?;
    write_bool(writer, false)?;
    write_var_i32(writer, 0)?;
    write_bool(writer, true)
}

pub fn write_player_abilities_packet<W: Write>(writer: &mut W, game_mode: GameMode) -> io::Result<()> {
    let flags = match game_mode {
        GameMode::Survival | GameMode::Adventure => 0,
        GameMode::Creative => 0x0d,
        GameMode::Spectator => 0x0f,
    };
    writer.write_all(&[flags])?;
    writer.write_all(&0.05f32.to_be_bytes())?;
    writer.write_all(&0.1f32.to_be_bytes())
}

pub fn write_command_suggestions_response<R: Read>(
    stream: &mut TcpStream,
    compression: CompressionState,
    input: &mut R,
) -> io::Result<()> {
    let transaction_id = read_var_i32(input)?;
    let command = read_string(input, 32767)?;
    let query = command.strip_prefix('/').unwrap_or(&command);
    let matches: Vec<&str> = PLAY_COMMAND_SUGGESTIONS
        .iter()
        .copied()
        .filter(|candidate| candidate.starts_with(query))
        .collect();
    let replacement_start = if command.starts_with('/') { 1 } else { 0 };

    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_COMMAND_SUGGESTIONS_PACKET_ID,
        |payload| {
            write_var_i32(payload, transaction_id)?;
            write_var_i32(payload, replacement_start)?;
            write_var_i32(payload, query.len() as i32)?;
            write_var_i32(payload, matches.len() as i32)?;
            for candidate in matches {
                write_string(payload, candidate)?;
                write_bool(payload, false)?;
            }
            Ok(())
        },
    )
}

pub fn uuid_from_hyphenated(value: &str) -> io::Result<Uuid> {
    let hex: String = value.chars().filter(|ch| *ch != '-').collect();
    if hex.len() != 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid UUID length",
        ));
    }
    let mut bytes = [0u8; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        let start = index * 2;
        *byte = u8::from_str_radix(&hex[start..start + 2], 16)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
    }
    Ok(Uuid(bytes))
}

pub fn write_initialize_world_border_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    writer.write_all(&0.0f64.to_be_bytes())?;
    writer.write_all(&0.0f64.to_be_bytes())?;
    writer.write_all(&59_999_968.0f64.to_be_bytes())?;
    writer.write_all(&59_999_968.0f64.to_be_bytes())?;
    write_var_i64(writer, 0)?;
    write_var_i32(writer, 29_999_984)?;
    write_var_i32(writer, 5)?;
    write_var_i32(writer, 15)
}

pub fn write_default_spawn_position_packet<W: Write>(
    writer: &mut W,
    x: i32,
    y: i32,
    z: i32,
) -> io::Result<()> {
    write_identifier(writer, &Identifier::parse("minecraft:overworld").unwrap())?;
    writer.write_all(&block_pos_as_long(x, y, z).to_be_bytes())?;
    writer.write_all(&0.0f32.to_be_bytes())?;
    writer.write_all(&0.0f32.to_be_bytes())
}

pub fn block_pos_as_long(x: i32, y: i32, z: i32) -> i64 {
    const PACKED_HORIZONTAL_LENGTH: u32 = 26;
    const PACKED_Y_LENGTH: u32 = 12;
    const Z_OFFSET: u32 = PACKED_Y_LENGTH;
    const X_OFFSET: u32 = PACKED_Y_LENGTH + PACKED_HORIZONTAL_LENGTH;
    const PACKED_X_MASK: i64 = (1_i64 << PACKED_HORIZONTAL_LENGTH) - 1;
    const PACKED_Y_MASK: i64 = (1_i64 << PACKED_Y_LENGTH) - 1;
    const PACKED_Z_MASK: i64 = (1_i64 << PACKED_HORIZONTAL_LENGTH) - 1;

    ((x as i64 & PACKED_X_MASK) << X_OFFSET)
        | (y as i64 & PACKED_Y_MASK)
        | ((z as i64 & PACKED_Z_MASK) << Z_OFFSET)
}

pub fn write_generated_spawn_chunk_packet<W: Write>(
    writer: &mut W,
    x: i32,
    z: i32,
    world_root: &Path,
    world_seed: i64,
) -> io::Result<()> {
    let chunk = load_or_generate_spawn_chunk_uncached(x, z, world_root, world_seed);
    write_generated_spawn_chunk_payload(writer, &chunk)
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedChunkEntitySpawnPlan {
    pub add_entity: ClientboundAddEntityPacket,
    pub metadata: Option<ClientboundSetEntityDataPacket>,
}

pub fn write_generated_chunk_entity_spawn_packets<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    plan: &GeneratedChunkEntitySpawnPlan,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID,
        |_| Ok(()),
    )?;
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_ADD_ENTITY_PACKET_ID,
        |payload| plan.add_entity.write(payload),
    )?;
    if let Some(metadata) = &plan.metadata {
        write_framed_packet_with_compression(
            writer,
            compression,
            CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID,
            |payload| metadata.write(payload),
        )?;
    }
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID,
        |_| Ok(()),
    )
}

pub fn write_generated_spawn_chunk_payload<W: Write>(
    writer: &mut W,
    chunk: &LevelChunk,
) -> io::Result<()> {
    let started = Instant::now();
    let light_data = ClientboundLightUpdatePacketData::from_chunk(&chunk);
    let light_ms = started.elapsed().as_millis();
    let packet_started = Instant::now();
    let packet = ClientboundLevelChunkWithLightPacket::from_chunk(&chunk, light_data);
    let packet_build_ms = packet_started.elapsed().as_millis();
    let write_started = Instant::now();
    let result = write_level_chunk_with_light_payload(writer, &packet);
    eprintln!(
        "[worldgen] chunk=({}, {}) packet light={}ms build={}ms write={}ms bytes={} block_entities={} heightmaps={}",
        chunk.pos.x,
        chunk.pos.z,
        light_ms,
        packet_build_ms,
        write_started.elapsed().as_millis(),
        packet
            .chunk_data
            .as_ref()
            .map(|data| data.buffer.len())
            .unwrap_or(0),
        packet
            .chunk_data
            .as_ref()
            .map(|data| data.block_entities.len())
            .unwrap_or(0),
        packet
            .chunk_data
            .as_ref()
            .map(|data| data.heightmaps.len())
            .unwrap_or(0)
    );
    result
}

pub fn load_or_generate_spawn_chunk_uncached(
    x: i32,
    z: i32,
    world_root: &Path,
    world_seed: i64,
) -> LevelChunk {
    let started = Instant::now();
    let pos = ChunkPos { x, z };
    let region_dir = world_root.join("region");
    let mut source = "region";
    let region_started = Instant::now();
    let loaded = try_load_chunk_from_region(&region_dir, pos);
    let region_ms = region_started.elapsed().as_millis();
    let chunk = loaded.unwrap_or_else(|| {
        source = match live_chunk_generation_mode() {
            LiveChunkGenerationMode::Preview => "generated-preview",
            LiveChunkGenerationMode::RealSurface => "generated-real-surface",
        };
        match generate_overworld_spawn_chunk_for_preset_with_mode_timed(
            pos,
            "normal",
            live_chunk_generation_mode(),
            world_seed,
            true,
        ) {
            Ok((chunk, timings)) => {
                eprintln!(
                    "[worldgen] chunk=({}, {}) phases region={}ms preset={}ms terrain={}ms fill={}ms fill_init_sections={}ms fill_noise_chunk_init={}ms fill_aquifer_init={}ms fill_block_loop={}ms fill_density_lookup={}us fill_aquifer_compute={}us fill_ore_vein_lookup={}us fill_ore_decision={}us fill_interpolation_update={}us interpolators={} fill_full_noise_cache={}ms fill_full_noise_cache_fills={} fill_vein_noise_cache={}ms fill_vein_noise_cache_fills={} cache_once_scalar_hits={} cache_once_scalar_misses={} cache_once_array_hits={} cache_once_array_misses={} fill_heightmap_pack={}ms fill_cell_columns={} fill_block_samples={} fill_block_writes={} aquifer_calls={} ore_vein_samples={} surface={}ms surface_noise_setup={}ms surface_prelim={}ms surface_column_loop={}ms surface_columns={} surface_block_samples={} surface_block_writes={} tree_context={}ms tree_context_chunks={} tree_decoration={}ms tree_blocks={} heightmaps={}ms heightmap_decode={}ms heightmap_scan={}ms heightmap_pack={}ms heightmap_sections={} heightmap_samples={} mobs={}ms mob_plan={}ms mob_biome={}ms mob_spawn_plan={}ms mob_apply={}ms mob_top={}ms mob_position_ok={}ms mob_snap_collision={}ms mob_rules={}ms mob_queue={}ms mob_random_walk={}ms mob_batches={} mob_attempts={} mobs_spawned={}",
                    x,
                    z,
                    region_ms,
                    timings.resolve_preset_ms,
                    timings.terrain_ms,
                    timings.terrain.fill_total_ms,
                    timings.terrain.fill_init_sections_ms,
                    timings.terrain.fill_noise_chunk_init_ms,
                    timings.terrain.fill_aquifer_init_ms,
                    timings.terrain.fill_block_loop_ms,
                    timings.terrain.fill_density_lookup_us,
                    timings.terrain.fill_aquifer_compute_us,
                    timings.terrain.fill_ore_vein_lookup_us,
                    timings.terrain.fill_ore_decision_us,
                    timings.terrain.fill_interpolation_update_us,
                    timings.terrain.interpolator_count,
                    timings.terrain.fill_full_noise_cache_ms,
                    timings.terrain.full_noise_cache_fills,
                    timings.terrain.fill_vein_noise_cache_ms,
                    timings.terrain.vein_noise_cache_fills,
                    timings.terrain.cache_once_scalar_hits,
                    timings.terrain.cache_once_scalar_misses,
                    timings.terrain.cache_once_array_hits,
                    timings.terrain.cache_once_array_misses,
                    timings.terrain.fill_heightmap_pack_ms,
                    timings.terrain.cell_columns,
                    timings.terrain.block_samples,
                    timings.terrain.block_writes,
                    timings.terrain.aquifer_calls,
                    timings.terrain.ore_vein_samples,
                    timings.terrain.surface_total_ms,
                    timings.terrain.surface_noise_setup_ms,
                    timings.terrain.surface_prelim_ms,
                    timings.terrain.surface_column_loop_ms,
                    timings.terrain.surface_columns,
                    timings.terrain.surface_block_samples,
                    timings.terrain.surface_block_writes,
                    timings.tree_context_ms,
                    timings.tree_context_chunks,
                    timings.tree_decoration_ms,
                    timings.tree_blocks,
                    timings.heightmaps.total_ms,
                    timings.heightmaps.decode_sections_ms,
                    timings.heightmaps.scan_blocks_ms,
                    timings.heightmaps.pack_store_ms,
                    timings.heightmaps.sections_decoded,
                    timings.heightmaps.block_samples,
                    timings.mobs.total_ms,
                    timings.mobs.plan_ms,
                    timings.mobs.biome_ms,
                    timings.mobs.spawn_plan_ms,
                    timings.mobs.apply_batches_ms,
                    timings.mobs.top_position_ms,
                    timings.mobs.position_ok_ms,
                    timings.mobs.snap_collision_ms,
                    timings.mobs.spawn_rules_ms,
                    timings.mobs.queue_ms,
                    timings.mobs.random_walk_ms,
                    timings.mobs.batches,
                    timings.mobs.attempts,
                    timings.mobs.mobs_spawned
                );
                chunk
            }
            Err(err) => {
                source = "generated-fallback-empty";
                eprintln!(
                    "[worldgen] chunk=({}, {}) generation failed after {}ms: {}",
                    x,
                    z,
                    started.elapsed().as_millis(),
                    err
                );
                crate::storage::chunk::LevelChunk::empty(pos)
            }
        }
    });
    eprintln!(
        "[worldgen] chunk=({}, {}) source={} status={} sections={} elapsed={}ms",
        x,
        z,
        source,
        chunk.status,
        chunk.sections.len(),
        started.elapsed().as_millis()
    );
    chunk
}

pub fn live_chunk_generation_mode() -> LiveChunkGenerationMode {
    match std::env::var("RUSTCRAFT_WORLDGEN").as_deref() {
        Ok("preview") => LiveChunkGenerationMode::Preview,
        Ok("real-surface") | Ok("surface") | Err(_) => LiveChunkGenerationMode::RealSurface,
        Ok(other) => {
            eprintln!(
                "unknown RUSTCRAFT_WORLDGEN={other:?}; using real-surface (set preview for scaffold terrain)"
            );
            LiveChunkGenerationMode::RealSurface
        }
    }
}

pub fn generated_chunk_entity_add_packets(chunk: &LevelChunk) -> Vec<ClientboundAddEntityPacket> {
    generated_chunk_entity_spawn_plans(chunk)
        .into_iter()
        .map(|plan| plan.add_entity)
        .collect()
}

pub fn generated_chunk_entity_spawn_plans(chunk: &LevelChunk) -> Vec<GeneratedChunkEntitySpawnPlan> {
    chunk
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| generated_chunk_entity_spawn_plan(chunk.pos, index, entity))
        .collect()
}

pub fn generated_chunk_entity_spawn_plan(
    chunk_pos: ChunkPos,
    index: usize,
    entity: &Tag,
) -> Option<GeneratedChunkEntitySpawnPlan> {
    let Tag::Compound(fields) = entity else {
        return None;
    };
    let entity_type_name = tag_string_field(fields, "id")?;
    let entity_type = generated_mob_entity_type_network_id(entity_type_name)?;
    let uuid = uuid_from_hyphenated(tag_string_field(fields, "UUID")?).ok()?;
    let [x, y, z] = tag_double_triplet_field(fields, "Pos")?;
    let [yaw, pitch] = tag_float_pair_field(fields, "Rotation")?;
    let runtime_id = generated_chunk_entity_runtime_id(chunk_pos, index);
    let add_entity = ClientboundAddEntityPacket::new(
        runtime_id,
        uuid,
        entity_type,
        Vec3 { x, y, z },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        (pitch, yaw),
        yaw,
        0,
    );
    Some(GeneratedChunkEntitySpawnPlan {
        add_entity,
        metadata: generated_chunk_entity_metadata_packet(runtime_id, entity_type_name, fields),
    })
}

pub fn generated_chunk_entity_metadata_packet(
    runtime_id: i32,
    entity_type: &str,
    fields: &[(String, Tag)],
) -> Option<ClientboundSetEntityDataPacket> {
    let mut packed_items = Vec::new();
    match entity_type {
        "minecraft:cat" => {
            if let Some(variant) = tag_string_field(fields, "variant")
                .and_then(cat_variant_registry_id)
                .filter(|variant| *variant != 1)
            {
                packed_items.push(
                    EntityDataValue::typed(20, EntityMetadataValue::CatVariant(variant)).ok()?,
                );
            }
            if let Some(sound_variant) = tag_string_field(fields, "sound_variant")
                .and_then(cat_sound_variant_registry_id)
                .filter(|sound_variant| *sound_variant != 0)
            {
                packed_items.push(
                    EntityDataValue::typed(24, EntityMetadataValue::CatSoundVariant(sound_variant))
                        .ok()?,
                );
            }
        }
        "minecraft:chicken" => {
            if let Some(variant) = tag_string_field(fields, "variant")
                .and_then(chicken_variant_registry_id)
                .filter(|variant| *variant != 1)
            {
                packed_items.push(
                    EntityDataValue::typed(18, EntityMetadataValue::ChickenVariant(variant))
                        .ok()?,
                );
            }
            if let Some(sound_variant) = tag_string_field(fields, "sound_variant")
                .and_then(chicken_sound_variant_registry_id)
                .filter(|sound_variant| *sound_variant != 0)
            {
                packed_items.push(
                    EntityDataValue::typed(
                        19,
                        EntityMetadataValue::ChickenSoundVariant(sound_variant),
                    )
                    .ok()?,
                );
            }
        }
        "minecraft:cow" => {
            if let Some(variant) = tag_string_field(fields, "variant")
                .and_then(cow_variant_registry_id)
                .filter(|variant| *variant != 1)
            {
                packed_items.push(
                    EntityDataValue::typed(18, EntityMetadataValue::CowVariant(variant)).ok()?,
                );
            }
            if let Some(sound_variant) = tag_string_field(fields, "sound_variant")
                .and_then(cow_sound_variant_registry_id)
                .filter(|sound_variant| *sound_variant != 0)
            {
                packed_items.push(
                    EntityDataValue::typed(19, EntityMetadataValue::CowSoundVariant(sound_variant))
                        .ok()?,
                );
            }
        }
        "minecraft:frog" => {
            if let Some(variant) = tag_string_field(fields, "variant")
                .and_then(frog_variant_registry_id)
                .filter(|variant| *variant != 1)
            {
                packed_items.push(
                    EntityDataValue::typed(18, EntityMetadataValue::FrogVariant(variant)).ok()?,
                );
            }
        }
        "minecraft:pig" => {
            if let Some(variant) = tag_string_field(fields, "variant")
                .and_then(pig_variant_registry_id)
                .filter(|variant| *variant != 1)
            {
                packed_items.push(
                    EntityDataValue::typed(19, EntityMetadataValue::PigVariant(variant)).ok()?,
                );
            }
            if let Some(sound_variant) = tag_string_field(fields, "sound_variant")
                .and_then(pig_sound_variant_registry_id)
                .filter(|sound_variant| *sound_variant != 1)
            {
                packed_items.push(
                    EntityDataValue::typed(20, EntityMetadataValue::PigSoundVariant(sound_variant))
                        .ok()?,
                );
            }
        }
        "minecraft:wolf" => {
            if let Some(variant) = tag_string_field(fields, "variant")
                .and_then(wolf_variant_registry_id)
                .filter(|variant| *variant != 3)
            {
                packed_items.push(
                    EntityDataValue::typed(23, EntityMetadataValue::WolfVariant(variant)).ok()?,
                );
            }
            if let Some(sound_variant) = tag_string_field(fields, "sound_variant")
                .and_then(wolf_sound_variant_registry_id)
                .filter(|sound_variant| *sound_variant != 2)
            {
                packed_items.push(
                    EntityDataValue::typed(
                        24,
                        EntityMetadataValue::WolfSoundVariant(sound_variant),
                    )
                    .ok()?,
                );
            }
        }
        "minecraft:zombie_nautilus" => {
            if let Some(variant) = tag_string_field(fields, "variant")
                .and_then(zombie_nautilus_variant_registry_id)
                .filter(|variant| *variant != 0)
            {
                packed_items.push(
                    EntityDataValue::typed(21, EntityMetadataValue::ZombieNautilusVariant(variant))
                        .ok()?,
                );
            }
        }
        _ => {}
    }
    (!packed_items.is_empty()).then_some(ClientboundSetEntityDataPacket {
        id: runtime_id,
        packed_items,
    })
}

pub fn tag_string_field<'a>(fields: &'a [(String, Tag)], name: &str) -> Option<&'a str> {
    fields.iter().find_map(|(field_name, value)| {
        if field_name == name {
            if let Tag::String(value) = value {
                return Some(value.as_str());
            }
        }
        None
    })
}

pub fn resource_path_id(value: &str) -> &str {
    value.strip_prefix("minecraft:").unwrap_or(value)
}

pub fn cat_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "all_black" => Some(0),
        "black" => Some(1),
        "british_shorthair" => Some(2),
        "calico" => Some(3),
        "jellie" => Some(4),
        "persian" => Some(5),
        "ragdoll" => Some(6),
        "red" => Some(7),
        "siamese" => Some(8),
        "tabby" => Some(9),
        "white" => Some(10),
        _ => None,
    }
}

pub fn cat_sound_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "classic" => Some(0),
        "royal" => Some(1),
        _ => None,
    }
}

pub fn chicken_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "cold" => Some(0),
        "temperate" => Some(1),
        "warm" => Some(2),
        _ => None,
    }
}

pub fn chicken_sound_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "classic" => Some(0),
        "picky" => Some(1),
        _ => None,
    }
}

pub fn cow_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "cold" => Some(0),
        "temperate" => Some(1),
        "warm" => Some(2),
        _ => None,
    }
}

pub fn cow_sound_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "classic" => Some(0),
        "moody" => Some(1),
        _ => None,
    }
}

