use super::*;

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

#[cfg(test)]
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

#[cfg(test)]
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

pub fn write_player_abilities_packet<W: Write>(
    writer: &mut W,
    game_mode: GameMode,
) -> io::Result<()> {
    let flags = match game_mode {
        GameMode::Survival | GameMode::Adventure => 0,
        GameMode::Creative => 0x0d,
        GameMode::Spectator => 0x0f,
    };
    writer.write_all(&[flags])?;
    writer.write_all(&0.05f32.to_be_bytes())?;
    writer.write_all(&0.1f32.to_be_bytes())
}

pub fn write_command_suggestions_response<W: Write, R: Read>(
    stream: &mut W,
    compression: CompressionState,
    input: &mut R,
) -> io::Result<()> {
    let packet = ServerboundCommandSuggestionPacket::read(input)?;
    let command = packet.command;
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
            write_var_i32(payload, packet.id)?;
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

pub fn handle_chat_packet<R: Read>(
    stream: &mut TcpStream,
    compression: CompressionState,
    input: &mut R,
    profile: &NameAndId,
) -> io::Result<()> {
    // Java ServerGamePacketListenerImpl.handleChat delegates to tryHandleChat,
    // which rejects StringUtil-disallowed chat characters before decoration.
    let packet = ServerboundChatPacket::read(input)?;
    if chat_message_is_illegal(&packet.message) {
        write_disconnect_component(
            stream,
            compression,
            "multiplayer.disconnect.illegal_characters",
        )?;
        return Ok(());
    }

    write_system_chat_text(
        stream,
        compression,
        &format!("<{}> {}", profile.name, packet.message),
        false,
    )
}

pub struct ChatCommandContext<'a> {
    pub profile: &'a NameAndId,
    pub play_state: &'a mut PlaySessionState,
    pub properties: &'a ServerProperties,
    pub player_access: &'a Arc<Mutex<PlayerAccess>>,
    pub world_seed: i64,
}

pub fn handle_chat_command_packet<R: Read>(
    stream: &mut TcpStream,
    compression: CompressionState,
    input: &mut R,
    signed: bool,
    context: ChatCommandContext<'_>,
) -> io::Result<()> {
    let command = if signed {
        ServerboundChatCommandSignedPacket::read(input)?.command
    } else {
        ServerboundChatCommandPacket::read(input)?.command
    };

    // Java runs commands through the same chat validation gate with isCommand=true.
    if chat_message_is_illegal(&command) {
        write_disconnect_component(
            stream,
            compression,
            "multiplayer.disconnect.illegal_characters",
        )?;
        return Ok(());
    }

    let permissions =
        player_permission_set(context.profile, context.properties, context.player_access);
    let mut command_state = command_state_for_player(
        context.profile,
        context.play_state,
        context.properties,
        context.world_seed,
    );
    let result = execute_builtin_command(&mut command_state, permissions, &command);
    apply_command_side_effects(
        stream,
        compression,
        context.play_state,
        context.profile,
        &command_state,
    )?;
    match result {
        Ok(result) => write_system_chat_text(
            stream,
            compression,
            &command_feedback_text(&result, &command_state),
            false,
        ),
        Err(error) => write_system_chat_text(
            stream,
            compression,
            &format!("Command failed: {error:?}"),
            false,
        ),
    }
}

pub fn chat_message_is_illegal(message: &str) -> bool {
    message
        .chars()
        .any(|ch| ch == '\u{00a7}' || ch < ' ' || ch == '\u{7f}')
}

fn player_permission_set(
    profile: &NameAndId,
    properties: &ServerProperties,
    player_access: &Arc<Mutex<PlayerAccess>>,
) -> LevelBasedPermissionSet {
    let op_level = player_access
        .lock()
        .ok()
        .and_then(|access| access.op_level(&profile.uuid))
        .map(u32::from)
        .unwrap_or(0);
    let level = op_level.max(if op_level > 0 {
        properties.op_permission_level
    } else {
        0
    });
    LevelBasedPermissionSet::new(match level {
        4.. => PermissionLevel::Owners,
        3 => PermissionLevel::Admins,
        2 => PermissionLevel::Gamemasters,
        1 => PermissionLevel::Moderators,
        _ => PermissionLevel::All,
    })
}

fn command_state_for_player(
    profile: &NameAndId,
    play_state: &PlaySessionState,
    properties: &ServerProperties,
    world_seed: i64,
) -> ServerCommandState {
    let mut state = ServerCommandState {
        command_source_player: Some(profile.clone()),
        command_source_position: crate::command::Vec3 {
            x: play_state.x,
            y: play_state.y,
            z: play_state.z,
        },
        world_preset: properties.level_type.clone(),
        online_players: vec![profile.clone()],
        max_players: properties.max_players,
        world_seed,
        function_permission_level: function_permission_level_from_properties(properties),
        ..ServerCommandState::default()
    };
    state
        .player_game_modes
        .push(crate::command::PlayerGameMode {
            player: profile.clone(),
            gamemode: command_game_mode(play_state.game_mode),
        });
    state
}

pub fn function_permission_level_from_properties(properties: &ServerProperties) -> PermissionLevel {
    PermissionLevel::by_id(properties.function_permission_level)
}

fn apply_command_side_effects(
    stream: &mut TcpStream,
    compression: CompressionState,
    play_state: &mut PlaySessionState,
    profile: &NameAndId,
    command_state: &ServerCommandState,
) -> io::Result<()> {
    if let Some(entry) = command_state
        .player_game_modes
        .iter()
        .find(|entry| entry.player.uuid == profile.uuid)
    {
        let new_game_mode = play_game_mode(entry.gamemode);
        if new_game_mode != play_state.game_mode {
            play_state.previous_game_mode = Some(play_state.game_mode);
            play_state.game_mode = new_game_mode;
            play_state.abilities.apply_game_mode(play_state.game_mode);
            // Java ServerPlayer.setGameMode sends CHANGE_GAME_MODE followed by abilities.
            write_game_event(stream, compression, 3, play_state.game_mode as i32 as f32)?;
            write_framed_packet_with_compression(
                stream,
                compression,
                CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID,
                |payload| write_player_abilities_packet(payload, play_state.game_mode),
            )?;
        }
    }
    Ok(())
}

fn command_game_mode(game_mode: GameMode) -> crate::command::GameMode {
    match game_mode {
        GameMode::Survival => crate::command::GameMode::Survival,
        GameMode::Creative => crate::command::GameMode::Creative,
        GameMode::Adventure => crate::command::GameMode::Adventure,
        GameMode::Spectator => crate::command::GameMode::Spectator,
    }
}

fn play_game_mode(game_mode: crate::command::GameMode) -> GameMode {
    match game_mode {
        crate::command::GameMode::Survival => GameMode::Survival,
        crate::command::GameMode::Creative => GameMode::Creative,
        crate::command::GameMode::Adventure => GameMode::Adventure,
        crate::command::GameMode::Spectator => GameMode::Spectator,
    }
}

pub(super) fn command_feedback_text(
    result: &crate::command::CommandResult,
    state: &ServerCommandState,
) -> String {
    match result.feedback_key {
        "commands.seed.success" => format!("Seed: {}", state.world_seed),
        "commands.list.players" => format!(
            "There are {} of a max of {} players online: {}",
            state.online_players.len(),
            state.max_players,
            state
                .online_players
                .iter()
                .map(|player| player.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        "commands.gamemode.success.self" => "Set own game mode".to_string(),
        "commands.say.success" => "Message sent".to_string(),
        // RustCraft-only debug command feedback; vanilla has no `/biome` command.
        "commands.rustcraft.debug.biome" => {
            format!("Biome: {}", debug_biome_at_command_source(state))
        }
        key => format!("{key} ({})", result.success_count),
    }
}

pub fn write_system_chat_text(
    stream: &mut TcpStream,
    compression: CompressionState,
    text: &str,
    overlay: bool,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SYSTEM_CHAT_PACKET_ID,
        |payload| {
            ClientboundSystemChatPacket {
                content: literal_component_tag(text),
                overlay,
            }
            .write(payload)
        },
    )
}

fn write_disconnect_component(
    stream: &mut TcpStream,
    compression: CompressionState,
    translation_key: &str,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_DISCONNECT_PACKET_ID,
        |payload| {
            ClientboundDisconnectPacket {
                reason: ComponentJson(format!("{{\"translate\":\"{translation_key}\"}}")),
            }
            .write(payload)
        },
    )?;
    let _ = stream.shutdown(Shutdown::Both);
    Ok(())
}

pub fn literal_component_tag(text: &str) -> Tag {
    Tag::Compound(vec![("text".to_string(), Tag::String(text.to_string()))])
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
    write_identifier(writer, &parse_builtin_identifier("minecraft:overworld")?)?;
    writer.write_all(&block_pos_as_long(x, y, z).to_be_bytes())?;
    writer.write_all(&0.0f32.to_be_bytes())?;
    writer.write_all(&0.0f32.to_be_bytes())
}

fn parse_builtin_identifier(value: &'static str) -> io::Result<Identifier> {
    Identifier::parse(value).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("built-in identifier {value:?} failed to parse: {err}"),
        )
    })
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

#[cfg(test)]
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
    let light_data = ClientboundLightUpdatePacketData::from_chunk(chunk);
    let light_ms = started.elapsed().as_millis();
    let packet_started = Instant::now();
    let packet = ClientboundLevelChunkWithLightPacket::from_chunk(chunk, light_data);
    let packet_build_ms = packet_started.elapsed().as_millis();
    let write_started = Instant::now();
    let result = write_level_chunk_with_light_payload(writer, &packet);
    eprintln!(
        "[worldgen] chunk=({}, {}) packet light={}ms build={}ms write={}ms bytes={} block_entities={} heightmaps={} light_correct={}",
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
            .unwrap_or(0),
        chunk.light_correct,
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
    let mut chunk = loaded.unwrap_or_else(|| {
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
                log_generated_chunk_timings(x, z, region_ms, &timings);
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
    // Mirror Java's `ThreadedLevelLightEngine.initializeLight`/`lightChunk`:
    // if the chunk's per-section light arrays were not stamped at save time,
    // run the full propagator now so the outgoing packet ships real values
    // instead of the previous "fullbright everywhere" stub.
    let light_started = Instant::now();
    let light_result = if !chunk.light_correct {
        let level_height = crate::lighting::level_height::LevelHeightAccessor::new(
            crate::network::play::OVERWORLD_MIN_SECTION_Y * 16,
            crate::network::play::OVERWORLD_SECTION_COUNT as i32 * 16,
        );
        Some(crate::lighting::compute_chunk_lighting(
            &mut chunk,
            level_height,
            true,
        ))
    } else {
        None
    };
    let light_compute_ms = light_started.elapsed().as_millis();
    let (block_nodes, sky_nodes) = light_result
        .map(|r| (r.block_nodes_propagated, r.sky_nodes_propagated))
        .unwrap_or((0, 0));
    eprintln!(
        "[worldgen] chunk=({}, {}) source={} status={} sections={} elapsed={}ms light_compute={}ms light_block_nodes={} light_sky_nodes={}",
        x,
        z,
        source,
        chunk.status,
        chunk.sections.len(),
        started.elapsed().as_millis(),
        light_compute_ms,
        block_nodes,
        sky_nodes,
    );
    chunk
}

fn log_generated_chunk_timings(
    x: i32,
    z: i32,
    region_ms: u128,
    timings: &LiveChunkGenerationTimings,
) {
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

pub fn generated_chunk_entity_spawn_plans(
    chunk: &LevelChunk,
) -> Vec<GeneratedChunkEntitySpawnPlan> {
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
    let add_entity = ClientboundAddEntityPacket::new(AddEntityPacketInput {
        id: runtime_id,
        uuid,
        entity_type,
        position: Vec3 { x, y, z },
        movement: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        rotation: (pitch, yaw),
        y_head_rot: yaw,
        data: 0,
    });
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
        "minecraft:cat" => push_cat_metadata(fields, &mut packed_items)?,
        "minecraft:chicken" => push_chicken_metadata(fields, &mut packed_items)?,
        "minecraft:cow" => push_cow_metadata(fields, &mut packed_items)?,
        "minecraft:frog" => push_frog_metadata(fields, &mut packed_items)?,
        "minecraft:pig" => push_pig_metadata(fields, &mut packed_items)?,
        "minecraft:wolf" => push_wolf_metadata(fields, &mut packed_items)?,
        "minecraft:zombie_nautilus" => push_zombie_nautilus_metadata(fields, &mut packed_items)?,
        _ => {}
    }
    (!packed_items.is_empty()).then_some(ClientboundSetEntityDataPacket {
        id: runtime_id,
        packed_items,
    })
}

fn push_cat_metadata(
    fields: &[(String, Tag)],
    packed_items: &mut Vec<EntityDataValue>,
) -> Option<()> {
    if let Some(variant) = metadata_variant(fields, "variant", cat_variant_registry_id, 1) {
        packed_items
            .push(EntityDataValue::typed(20, EntityMetadataValue::CatVariant(variant)).ok()?);
    }
    if let Some(sound_variant) =
        metadata_variant(fields, "sound_variant", cat_sound_variant_registry_id, 0)
    {
        packed_items.push(
            EntityDataValue::typed(24, EntityMetadataValue::CatSoundVariant(sound_variant)).ok()?,
        );
    }
    Some(())
}

fn push_chicken_metadata(
    fields: &[(String, Tag)],
    packed_items: &mut Vec<EntityDataValue>,
) -> Option<()> {
    if let Some(variant) = metadata_variant(fields, "variant", chicken_variant_registry_id, 1) {
        packed_items
            .push(EntityDataValue::typed(18, EntityMetadataValue::ChickenVariant(variant)).ok()?);
    }
    if let Some(sound_variant) = metadata_variant(
        fields,
        "sound_variant",
        chicken_sound_variant_registry_id,
        0,
    ) {
        packed_items.push(
            EntityDataValue::typed(19, EntityMetadataValue::ChickenSoundVariant(sound_variant))
                .ok()?,
        );
    }
    Some(())
}

fn push_cow_metadata(
    fields: &[(String, Tag)],
    packed_items: &mut Vec<EntityDataValue>,
) -> Option<()> {
    if let Some(variant) = metadata_variant(fields, "variant", cow_variant_registry_id, 1) {
        packed_items
            .push(EntityDataValue::typed(18, EntityMetadataValue::CowVariant(variant)).ok()?);
    }
    if let Some(sound_variant) =
        metadata_variant(fields, "sound_variant", cow_sound_variant_registry_id, 0)
    {
        packed_items.push(
            EntityDataValue::typed(19, EntityMetadataValue::CowSoundVariant(sound_variant)).ok()?,
        );
    }
    Some(())
}

fn push_frog_metadata(
    fields: &[(String, Tag)],
    packed_items: &mut Vec<EntityDataValue>,
) -> Option<()> {
    if let Some(variant) = metadata_variant(fields, "variant", frog_variant_registry_id, 1) {
        packed_items
            .push(EntityDataValue::typed(18, EntityMetadataValue::FrogVariant(variant)).ok()?);
    }
    Some(())
}

fn push_pig_metadata(
    fields: &[(String, Tag)],
    packed_items: &mut Vec<EntityDataValue>,
) -> Option<()> {
    if let Some(variant) = metadata_variant(fields, "variant", pig_variant_registry_id, 1) {
        packed_items
            .push(EntityDataValue::typed(19, EntityMetadataValue::PigVariant(variant)).ok()?);
    }
    if let Some(sound_variant) =
        metadata_variant(fields, "sound_variant", pig_sound_variant_registry_id, 1)
    {
        packed_items.push(
            EntityDataValue::typed(20, EntityMetadataValue::PigSoundVariant(sound_variant)).ok()?,
        );
    }
    Some(())
}

fn push_wolf_metadata(
    fields: &[(String, Tag)],
    packed_items: &mut Vec<EntityDataValue>,
) -> Option<()> {
    if let Some(variant) = metadata_variant(fields, "variant", wolf_variant_registry_id, 3) {
        packed_items
            .push(EntityDataValue::typed(23, EntityMetadataValue::WolfVariant(variant)).ok()?);
    }
    if let Some(sound_variant) =
        metadata_variant(fields, "sound_variant", wolf_sound_variant_registry_id, 2)
    {
        packed_items.push(
            EntityDataValue::typed(24, EntityMetadataValue::WolfSoundVariant(sound_variant))
                .ok()?,
        );
    }
    Some(())
}

fn push_zombie_nautilus_metadata(
    fields: &[(String, Tag)],
    packed_items: &mut Vec<EntityDataValue>,
) -> Option<()> {
    let Some(variant) = metadata_variant(fields, "variant", zombie_nautilus_variant_registry_id, 0)
    else {
        return Some(());
    };
    packed_items.push(
        EntityDataValue::typed(21, EntityMetadataValue::ZombieNautilusVariant(variant)).ok()?,
    );
    Some(())
}

fn metadata_variant(
    fields: &[(String, Tag)],
    field_name: &str,
    registry_id: impl FnOnce(&str) -> Option<i32>,
    default_id: i32,
) -> Option<i32> {
    tag_string_field(fields, field_name)
        .and_then(registry_id)
        .filter(|variant| *variant != default_id)
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
