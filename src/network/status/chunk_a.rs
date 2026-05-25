use super::*;

/// Loads clock state from `{world_root}/server_clocks.json`.
/// Returns `None` on missing or malformed file; caller falls back to `ServerClockManager::default()`.
/// Java: ServerClockManager.TYPE SavedData — key "world_clocks"
pub fn load_server_clock_state(world_root: &Path) -> Option<ServerClockManager> {
    let path = world_root.join("server_clocks.json");
    let text = fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    let clock = |obj: &serde_json::Value| -> Option<crate::world_time::ClockInstance> {
        Some(crate::world_time::ClockInstance {
            total_ticks: obj["total_ticks"].as_i64()?,
            partial_tick: obj["partial_tick"].as_f64()? as f32,
            rate: obj["rate"].as_f64()? as f32,
            paused: obj["paused"].as_bool()?,
        })
    };
    Some(ServerClockManager {
        game_time: v["game_time"].as_i64()?,
        overworld: clock(&v["overworld"])?,
        the_end: clock(&v["the_end"])?,
    })
}

/// Saves clock state to `{world_root}/server_clocks.json`.
pub fn save_server_clock_state(world_root: &Path, manager: &ServerClockManager) {
    let clock_json = |c: &crate::world_time::ClockInstance| {
        serde_json::json!({
            "total_ticks": c.total_ticks,
            "partial_tick": c.partial_tick,
            "rate": c.rate,
            "paused": c.paused,
        })
    };
    let value = serde_json::json!({
        "game_time": manager.game_time,
        "overworld": clock_json(&manager.overworld),
        "the_end": clock_json(&manager.the_end),
    });
    let path = world_root.join("server_clocks.json");
    if let Ok(text) = serde_json::to_string_pretty(&value) {
        let _ = fs::write(path, text);
    }
}

/// Loads weather state from `{world_root}/server_weather.json`.
pub fn load_server_weather_state(world_root: &Path) -> Option<WeatherCycle> {
    let path = world_root.join("server_weather.json");
    let text = fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    let data = WeatherData {
        clear_weather_time: v["clear_weather_time"].as_i64()? as i32,
        rain_time: v["rain_time"].as_i64()? as i32,
        thunder_time: v["thunder_time"].as_i64()? as i32,
        raining: v["raining"].as_bool()?,
        thundering: v["thundering"].as_bool()?,
    };
    let mut cycle = WeatherCycle::new(data);
    cycle.rain_level = v["rain_level"].as_f64()? as f32;
    cycle.thunder_level = v["thunder_level"].as_f64()? as f32;
    cycle.old_rain_level = cycle.rain_level;
    cycle.old_thunder_level = cycle.thunder_level;
    Some(cycle)
}

/// Saves weather state to `{world_root}/server_weather.json`.
pub fn save_server_weather_state(world_root: &Path, cycle: &WeatherCycle) {
    let value = serde_json::json!({
        "clear_weather_time": cycle.data.clear_weather_time,
        "rain_time": cycle.data.rain_time,
        "thunder_time": cycle.data.thunder_time,
        "raining": cycle.data.raining,
        "thundering": cycle.data.thundering,
        "rain_level": cycle.rain_level,
        "thunder_level": cycle.thunder_level,
    });
    let path = world_root.join("server_weather.json");
    if let Ok(text) = serde_json::to_string_pretty(&value) {
        let _ = fs::write(path, text);
    }
}

/// Loads item entity state from `{world_root}/item_entities.json`.
///
/// Field names mirror Java's entity NBT format (`Pos`, `Motion`, `Age`, `PickupDelay`,
/// `Item`) so the file is human-readable and structurally close to the canonical
/// `entities/` region files used by the Java server.
///
/// Returns a default empty store if the file does not exist or cannot be parsed.
pub fn load_world_item_entities(world_root: &Path) -> WorldItemEntities {
    let path = world_root.join("item_entities.json");
    let Some(text) = fs::read_to_string(&path).ok() else {
        return WorldItemEntities::new();
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return WorldItemEntities::new();
    };
    let next_entity_id = v["NextEntityId"].as_i64().unwrap_or(1) as i32;
    let entities = v["Entities"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|e| {
                    let network_id = e["EntityNetworkId"].as_i64()? as i32;
                    let pos = e["Pos"].as_array()?;
                    let motion = e["Motion"].as_array()?;
                    let item_obj = e["Item"].as_object()?;
                    let item_name = item_static_name(item_obj.get("id")?.as_str()?)?;
                    let count = item_obj.get("count")?.as_i64()? as i32;
                    let age = e["Age"].as_i64().unwrap_or(0) as i32;
                    let pickup_delay = e["PickupDelay"].as_i64().unwrap_or(0) as i32;
                    let target_uuid = e["Owner"].as_str().map(|s| s.to_string());
                    Some(DroppedItem {
                        entity_id: network_id,
                        item: item_name,
                        count,
                        x: pos.first()?.as_f64()?,
                        y: pos.get(1)?.as_f64()?,
                        z: pos.get(2)?.as_f64()?,
                        vel_x: motion.first()?.as_f64()?,
                        vel_y: motion.get(1)?.as_f64()?,
                        vel_z: motion.get(2)?.as_f64()?,
                        age,
                        pickup_delay,
                        target_uuid,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    WorldItemEntities::restore(entities, next_entity_id)
}

/// Saves item entity state to `{world_root}/item_entities.json`.
///
/// Java: `EntityStorage.storeEntities()` / `ItemEntity.addAdditionalSaveData()`.
/// Field names match the canonical Java NBT names where applicable so the file is
/// recognisable to anyone familiar with the Java entity format.
pub fn save_world_item_entities(world_root: &Path, store: &WorldItemEntities) {
    let entities: Vec<serde_json::Value> = store
        .entities
        .iter()
        .map(|e| {
            let mut obj = serde_json::json!({
                "EntityNetworkId": e.entity_id,
                "id": "minecraft:item",
                "Pos": [e.x, e.y, e.z],
                "Motion": [e.vel_x, e.vel_y, e.vel_z],
                "Age": e.age,
                "PickupDelay": e.pickup_delay,
                "Item": {
                    "id": e.item,
                    "count": e.count,
                },
            });
            if let Some(owner) = &e.target_uuid {
                obj["Owner"] = serde_json::Value::String(owner.clone());
            }
            obj
        })
        .collect();
    let value = serde_json::json!({
        "NextEntityId": store.next_entity_id(),
        "Entities": entities,
    });
    let path = world_root.join("item_entities.json");
    if let Ok(text) = serde_json::to_string_pretty(&value) {
        let _ = fs::write(path, text);
    }
}

pub fn run_status_server(
    bind_ip: &str,
    port: u16,
    properties: &ServerProperties,
    world_root: &Path,
    world_seed: i64,
    console_input: &Receiver<ConsoleInput>,
) -> Result<(), String> {
    let address = format!("{bind_ip}:{port}");
    let listener = TcpListener::bind(&address)
        .map_err(|err| format!("Failed to bind status listener on {address}: {err}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|err| format!("Failed to configure status listener on {address}: {err}"))?;
    let favicon = load_favicon(Path::new("server-icon.png"))
        .map_err(|err| format!("Failed to load server-icon.png: {err}"))?;
    let active_logins = ActiveLoginRegistry::default();
    let world_root = Arc::new(world_root.to_path_buf());
    let chunk_cache = GeneratedChunkCache::default();
    // Async chunk generation coordinator (Phase 2/3 of the chunking rework).
    // Wired through every play-session entry point alongside `chunk_cache`,
    // but the legacy blocking batch path still calls `cache.get_or_load`
    // directly. The new per-tick drain in `run_play_loop` is what actually
    // consumes ready chunks from this pipeline; see CHECKLIST_CHUNKING_CHANGES.md.
    let chunk_pipeline_workers = thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(4)
        .clamp(2, 8);
    let chunk_pipeline = ChunkPipeline::new(
        chunk_cache.clone(),
        (*world_root).clone(),
        world_seed,
        chunk_pipeline_workers,
    );
    // Periodic chunk save loop. Java mirror: MinecraftServer's autosave
    // pass invoked from tickServer — block updates mutate the in-memory
    // chunk + set the unsaved flag; this thread is what actually pushes
    // the bytes to disk on a coarse interval. We use 30 s (vanilla
    // default is 5 min / `RustcraftDefault.WORLD_AUTOSAVE_INTERVAL`); a
    // shorter window keeps the disconnect-vs-save race tight without
    // making the writes themselves any more expensive.
    spawn_chunk_flush_thread(
        chunk_cache.clone(),
        Arc::clone(&world_root),
        Duration::from_secs(30),
    );
    let player_access = Arc::new(Mutex::new(
        PlayerAccess::load_from_dir(Path::new(".")).unwrap_or_else(|err| {
            eprintln!("status access file load error: {err}");
            PlayerAccess::default()
        }),
    ));

    // Load or initialise shared clock/weather/item-entity state.
    // Java: ServerClockManager.TYPE SavedData (key "world_clocks"), ServerLevel weather data,
    //       EntityStorage loads entities from per-chunk region files under <world>/entities/.
    let initial_clock = load_server_clock_state(&world_root).unwrap_or_default();
    let initial_weather = load_server_weather_state(&world_root)
        .unwrap_or_else(|| WeatherCycle::new(WeatherData::default()));
    let clock: Arc<Mutex<ServerClockManager>> = Arc::new(Mutex::new(initial_clock));
    let weather: Arc<Mutex<WeatherCycle>> = Arc::new(Mutex::new(initial_weather));
    // World-level item entity store.  Shared across all player sessions and persisted to
    // item_entities.json so items survive both player disconnects and server restarts.
    // Java: ServerLevel.entityStorage — entity lists belong to the world, not any connection.
    let world_items: Arc<Mutex<WorldItemEntities>> =
        Arc::new(Mutex::new(load_world_item_entities(&world_root)));

    // Load vanilla recipes once at startup and share via Arc.
    // Java: MinecraftServer.loadDataPacks() → RecipeManager.apply()
    let recipe_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("vanilla-data/data/minecraft/recipe");
    let recipe_manager = load_recipe_directory(&recipe_dir).unwrap_or_else(|err| {
        panic!(
            "failed to load bundled vanilla recipes from {}: {err}",
            recipe_dir.display()
        )
    });
    log_info(&format!(
        "loaded {} bundled vanilla recipes",
        recipe_manager.recipe_map().values().len()
    ));
    let recipe_manager: Arc<RecipeManagerModel> = Arc::new(recipe_manager);

    // Background tick thread: advances clocks and weather at 20 TPS.
    // Java: MinecraftServer.tickChildren() — clockManager.tick() + advanceWeatherCycle()
    {
        let clock_t = Arc::clone(&clock);
        let weather_t = Arc::clone(&weather);
        let world_root_t = Arc::clone(&world_root);
        let world_items_t = Arc::clone(&world_items);
        thread::spawn(move || {
            let mut scheduled = ScheduledTimeChanges::default();
            let mut next_tick = Instant::now() + SERVER_TICK_DURATION;
            let mut tick_count: u64 = 0;
            loop {
                let now = Instant::now();
                if now < next_tick {
                    thread::sleep(next_tick - now);
                }
                next_tick += SERVER_TICK_DURATION;
                tick_count += 1;

                // advance_time=true: no per-world gamerule access yet; always advance.
                clock_t.lock().unwrap().tick(true, &mut scheduled);

                // Advance weather. can_have_weather=true for overworld.
                weather_t
                    .lock()
                    .unwrap()
                    .advance(true, true, DEFAULT_WEATHER_DURATIONS);

                // Persist every ~5 minutes.
                // Java: MinecraftServer.saveEverything() — entities flushed via EntityStorage.
                if tick_count % PERSISTENCE_INTERVAL_TICKS == 0 {
                    save_server_clock_state(&world_root_t, &clock_t.lock().unwrap());
                    save_server_weather_state(&world_root_t, &weather_t.lock().unwrap());
                    save_world_item_entities(&world_root_t, &world_items_t.lock().unwrap());
                }
            }
        });
    }

    println!("Status listener bound to {address}");

    loop {
        if should_stop(console_input, &player_access) {
            println!("Status listener stopping");
            save_server_clock_state(&world_root, &clock.lock().unwrap());
            save_server_weather_state(&world_root, &weather.lock().unwrap());
            save_world_item_entities(&world_root, &world_items.lock().unwrap());
            break;
        }
        match listener.accept() {
            Ok((stream, peer_addr)) => {
                let properties = properties.clone();
                let favicon = favicon.clone();
                let active_logins = active_logins.clone();
                let chunk_cache = chunk_cache.clone();
                let chunk_pipeline = chunk_pipeline.clone();
                let world_root = Arc::clone(&world_root);
                let player_access = Arc::clone(&player_access);
                let clock = Arc::clone(&clock);
                let weather = Arc::clone(&weather);
                let recipe_manager = Arc::clone(&recipe_manager);
                let world_items = Arc::clone(&world_items);
                let remote_ip = peer_addr.ip().to_string();
                let remote_for_log = if properties.log_ips {
                    remote_ip.clone()
                } else {
                    "<redacted>".to_string()
                };
                thread::spawn(move || {
                    if let Err(err) = handle_status_connection(
                        stream,
                        &properties,
                        favicon.as_deref(),
                        &active_logins,
                        &chunk_cache,
                        &chunk_pipeline,
                        &player_access,
                        &world_root,
                        world_seed,
                        &remote_ip,
                        &clock,
                        &weather,
                        &recipe_manager,
                        &world_items,
                    ) {
                        eprintln!("status connection error from {remote_for_log}: {err}");
                    }
                });
            }
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(25));
            }
            Err(err) => eprintln!("status accept error: {err}"),
        }
    }

    Ok(())
}

pub fn should_stop(
    console_input: &Receiver<ConsoleInput>,
    player_access: &Arc<Mutex<PlayerAccess>>,
) -> bool {
    loop {
        match console_input.try_recv() {
            Ok(input) if input.line.eq_ignore_ascii_case("stop") => return true,
            Ok(input)
                if input.line.eq_ignore_ascii_case("reload")
                    || input.line.eq_ignore_ascii_case("whitelist reload") =>
            {
                match PlayerAccess::load_from_dir(Path::new(".")) {
                    Ok(reloaded) => {
                        if let Ok(mut access) = player_access.lock() {
                            *access = reloaded;
                            println!("Reloaded player access files");
                        } else {
                            eprintln!("status access reload error: player access lock poisoned");
                        }
                    }
                    Err(err) => eprintln!("status access reload error: {err}"),
                }
            }
            Ok(_) => {}
            Err(TryRecvError::Empty) => return false,
            Err(TryRecvError::Disconnected) => return false,
        }
    }
}

pub fn handle_status_connection(
    mut stream: TcpStream,
    properties: &ServerProperties,
    favicon: Option<&str>,
    active_logins: &ActiveLoginRegistry,
    chunk_cache: &GeneratedChunkCache,
    chunk_pipeline: &ChunkPipeline,
    player_access: &Arc<Mutex<PlayerAccess>>,
    world_root: &Path,
    world_seed: i64,
    remote_ip: &str,
    clock: &Arc<Mutex<ServerClockManager>>,
    weather: &Arc<Mutex<WeatherCycle>>,
    recipe_manager: &RecipeManagerModel,
    world_items: &Arc<Mutex<WorldItemEntities>>,
) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;

    let mut first = [0u8; 1];
    if stream.peek(&mut first)? == 1 && first[0] == 0xFE {
        return handle_legacy_status_tcp_connection(&mut stream, properties);
    }

    let handshake = read_packet(&mut stream)?;
    let mut input = Cursor::new(handshake);
    let packet_id = read_var_i32(&mut input)?;
    if packet_id != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected handshake",
        ));
    }

    let protocol = read_var_i32(&mut input)?;
    let server_address = read_string(&mut input, 255)?;
    let mut port_bytes = [0u8; 2];
    input.read_exact(&mut port_bytes)?;
    let _server_port = u16::from_be_bytes(port_bytes);
    let next_state = read_var_i32(&mut input)?;
    if next_state == 2 {
        if protocol != PROTOCOL_VERSION {
            return write_login_protocol_mismatch_disconnect(&mut stream, protocol);
        }
        return handle_login_connection(
            &mut stream,
            properties,
            active_logins,
            chunk_cache,
            chunk_pipeline,
            player_access,
            world_root,
            world_seed,
            remote_ip,
            login_host_ip(&server_address),
            clock,
            weather,
            recipe_manager,
            world_items,
        );
    }
    if next_state != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported handshake target state",
        ));
    }
    if !properties.enable_status {
        return Ok(());
    }

    loop {
        let packet = read_packet(&mut stream)?;
        let mut input = Cursor::new(packet);
        match read_var_i32(&mut input)? {
            0 => {
                let json = status_json(properties, favicon);
                write_status_response_packet(&mut stream, &json)?;
            }
            1 => {
                let request = ServerboundPingRequestPacket::read(&mut input)?;
                write_status_pong_packet(&mut stream, request)?;
                return Ok(());
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "unknown status packet",
                ))
            }
        }
    }
}

pub fn write_login_protocol_mismatch_disconnect(
    stream: &mut TcpStream,
    protocol: i32,
) -> io::Result<()> {
    let key = if protocol < 754 {
        "multiplayer.disconnect.outdated_client"
    } else {
        "multiplayer.disconnect.incompatible"
    };
    write_framed_packet(stream, CLIENTBOUND_LOGIN_DISCONNECT_PACKET_ID, |payload| {
        ClientboundLoginDisconnectPacket {
            reason: crate::network::codec::ComponentJson(format!(
                "{{\"translate\":\"{}\",\"with\":[\"{}\"]}}",
                key, VERSION_NAME
            )),
        }
        .write(payload)
    })
}

pub fn login_compression_threshold(properties: &ServerProperties) -> Option<i32> {
    (properties.network_compression_threshold >= 0)
        .then_some(properties.network_compression_threshold)
}

pub fn handle_login_connection(
    stream: &mut TcpStream,
    properties: &ServerProperties,
    active_logins: &ActiveLoginRegistry,
    chunk_cache: &GeneratedChunkCache,
    chunk_pipeline: &ChunkPipeline,
    player_access: &Arc<Mutex<PlayerAccess>>,
    world_root: &Path,
    world_seed: i64,
    remote_ip: &str,
    login_host_ip: Option<String>,
    clock: &Arc<Mutex<ServerClockManager>>,
    weather: &Arc<Mutex<WeatherCycle>>,
    recipe_manager: &RecipeManagerModel,
    world_items: &Arc<Mutex<WorldItemEntities>>,
) -> io::Result<()> {
    let packet = read_packet(stream)?;
    let mut input = Cursor::new(packet);
    let packet_id = read_var_i32(&mut input)?;
    if packet_id != SERVERBOUND_HELLO_PACKET_ID {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected login hello",
        ));
    }

    let mut login = LoginSession::default();
    let finished = login.accept_offline_hello(ServerboundHelloPacket::read(&mut input)?);
    if let Some(reason) = login_access_disconnect_reason(
        properties,
        player_access,
        &finished.profile,
        remote_ip,
        login_host_ip.as_deref(),
    )? {
        return write_framed_packet(stream, CLIENTBOUND_LOGIN_DISCONNECT_PACKET_ID, |payload| {
            ClientboundLoginDisconnectPacket {
                reason: crate::network::codec::ComponentJson(format!(
                    "{{\"translate\":\"{reason}\"}}"
                )),
            }
            .write(payload)
        });
    }
    let (_active_login, replaced_stream) =
        active_logins.register_replacing(&finished.profile.uuid, stream)?;
    if let Some(replaced_stream) = replaced_stream {
        let _ = replaced_stream.shutdown(Shutdown::Both);
    }
    cache_login_profile(player_access, &finished.profile)?;
    let mut compression = CompressionState::disabled();
    if let Some(threshold) = login_compression_threshold(properties) {
        write_framed_packet(stream, CLIENTBOUND_LOGIN_COMPRESSION_PACKET_ID, |payload| {
            ClientboundLoginCompressionPacket {
                compression_threshold: threshold,
            }
            .write(payload)
        })?;
        login.set_compression(threshold);
        compression = CompressionState::enabled(threshold);
    }
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_LOGIN_FINISHED_PACKET_ID,
        |payload| finished.write(payload),
    )?;

    let packet = read_packet_with_compression(stream, compression)?;
    let mut input = Cursor::new(packet);
    let packet_id = read_var_i32(&mut input)?;
    if packet_id != SERVERBOUND_LOGIN_ACKNOWLEDGED_PACKET_ID {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected login acknowledgement",
        ));
    }
    login.acknowledge(ServerboundLoginAcknowledgedPacket::read(&mut input)?);

    if let Some(packet) = bug_report_server_links_packet(properties) {
        write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_CONFIGURATION_SERVER_LINKS_PACKET_ID,
            |payload| packet.write(payload),
        )?;
    }
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_UPDATE_ENABLED_FEATURES_PACKET_ID,
        |payload| {
            write_var_i32(payload, 1)?;
            write_identifier(payload, &Identifier::parse("minecraft:vanilla").unwrap())
        },
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.BIOME uses Biome.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_minimal_biome_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.CHAT_TYPE uses ChatType.DIRECT_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_chat_type_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.TRIM_PATTERN uses TrimPattern.DIRECT_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_trim_pattern_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.TRIM_MATERIAL uses TrimMaterial.DIRECT_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_minimal_trim_material_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.WOLF_VARIANT uses WolfVariant.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_wolf_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.WOLF_SOUND_VARIANT uses WolfSoundVariant.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_wolf_sound_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.PIG_VARIANT uses PigVariant.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_pig_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.PIG_SOUND_VARIANT uses PigSoundVariant.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_pig_sound_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.FROG_VARIANT uses FrogVariant.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_frog_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.CAT_VARIANT uses CatVariant.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_cat_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.CAT_SOUND_VARIANT uses CatSoundVariant.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_cat_sound_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.COW_SOUND_VARIANT uses CowSoundVariant.DIRECT_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_cow_sound_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.COW_VARIANT uses CowVariant.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_cow_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.CHICKEN_SOUND_VARIANT uses ChickenSoundVariant.DIRECT_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_chicken_sound_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.CHICKEN_VARIANT uses ChickenVariant.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_chicken_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.ZOMBIE_NAUTILUS_VARIANT uses ZombieNautilusVariant.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_zombie_nautilus_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.PAINTING_VARIANT uses PaintingVariant.DIRECT_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_painting_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.DIMENSION_TYPE uses DimensionType.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_minimal_dimension_type_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.DAMAGE_TYPE uses DamageType.DIRECT_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_minimal_damage_type_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.BANNER_PATTERN uses BannerPattern.DIRECT_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_banner_pattern_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.JUKEBOX_SONG uses JukeboxSong.DIRECT_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_jukebox_song_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.INSTRUMENT uses Instrument.DIRECT_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_instrument_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java:125,160
    // Registries.WORLD_CLOCK uses WorldClock.DIRECT_CODEC (MapCodec.unitCodec — empty compound).
    // Must be sent before any ClientboundSetTimePacket so the client can resolve clock VarInt IDs.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_world_clock_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java:125,160
    // Registries.TIMELINE uses Timeline.NETWORK_CODEC (syncable tracks only).
    // Must be sent before the tags packet so timeline tag IDs can reference these entries.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_timeline_registry_packet,
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_UPDATE_TAGS_PACKET_ID,
        write_minimal_update_tags_packet,
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID,
        write_vanilla_known_packs_packet,
    )?;
    wait_for_configuration_packet(
        stream,
        compression,
        SERVERBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID,
        "selected known packs",
    )?;
    if let Some(code_of_conduct) = load_code_of_conduct_for_language(properties, "en_us")? {
        write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_CONFIGURATION_CODE_OF_CONDUCT_PACKET_ID,
            |payload| ClientboundCodeOfConductPacket { code_of_conduct }.write(payload),
        )?;
        wait_for_configuration_packet(
            stream,
            compression,
            SERVERBOUND_CONFIGURATION_ACCEPT_CODE_OF_CONDUCT_PACKET_ID,
            "code of conduct acceptance",
        )?;
    }
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_FINISH_PACKET_ID,
        |_payload| Ok(()),
    )?;

    wait_for_configuration_packet(
        stream,
        compression,
        SERVERBOUND_CONFIGURATION_FINISH_PACKET_ID,
        "finish configuration",
    )?;

    let mut play_state = load_play_session_state(
        world_root,
        &finished.profile.uuid,
        properties,
        recipe_manager.recipe_map(),
        world_seed,
    );

    // Snapshot current clock and weather state for the join packet.
    // Java: ServerClockManager.createFullSyncPacket() on player join, ServerLevel.sendLevelInfo()
    let (join_game_time, join_clock_data) = {
        let cm = clock.lock().unwrap();
        cm.full_sync_data(true)
    };
    let (join_rain_level, join_thunder_level) = {
        let wc = weather.lock().unwrap();
        (wc.rain_level, wc.thunder_level)
    };

    write_minimal_play_join(
        stream,
        compression,
        properties,
        world_seed,
        &finished.profile,
        &play_state,
        recipe_manager,
        world_root,
        join_game_time,
        join_clock_data,
        join_rain_level,
        join_thunder_level,
    )?;
    let mut current_chunk_x = chunk_coordinate(play_state.x);
    let mut current_chunk_z = chunk_coordinate(play_state.z);
    let chunk_batch_radius = chunk_batch_radius(properties);
    let mut loaded_chunks = chunk_window(current_chunk_x, current_chunk_z, chunk_batch_radius);
    // Per-session chunk sender (Java mirror: PlayerChunkSender attached to
    // ServerPlayer). Seeded with the initial view-distance window below;
    // the per-tick `drain_chunk_sender` call inside the play loop produces
    // the actual chunk batches once the pipeline has generated chunks.
    // `memory_connection=false` because this is a real socket-backed
    // connection — Java's memory-connection short-circuit (LAN integrated
    // servers) does not apply.
    let mut chunk_sender = PlayerChunkSender::new(false);
    let mut chunk_pipeline_stats = ChunkPipelineSessionStats::default();
    seed_chunk_window(
        &mut chunk_sender,
        chunk_pipeline,
        current_chunk_x,
        current_chunk_z,
        chunk_batch_radius,
    );
    stream.set_read_timeout(Some(SERVER_TICK_DURATION))?;
    let mut last_keep_alive = Instant::now();
    let mut keep_alive_id = 0_i64;
    // Track last sent weather levels so we can detect changes and notify the client.
    // Java: ServerLevel.advanceWeatherCycle() broadcasts RainLevelChange/ThunderLevelChange
    let mut last_sent_rain_level = join_rain_level;
    let mut last_sent_thunder_level = join_thunder_level;
    let mut last_time_sync = Instant::now();
    let mut rate_limiter =
        PacketRateLimiter::new(properties.rate_limit_packets_per_second, Instant::now());
    let world_layout = WorldLayout::new(world_root);

    // On login: re-send ADD_ENTITY + SET_ENTITY_DATA bundles for every item entity that
    // is already on the ground.  This mirrors Java's ServerEntity.addPairing() called during
    // ChunkMap.updatePlayerMobTypeMap() when a player enters tracking range of an entity.
    // Without this, items dropped before a disconnect are invisible after reconnecting.
    {
        let items = world_items.lock().unwrap();
        for item in &items.entities {
            if let Some(item_pid) = item_protocol_id(item.item) {
                write_item_entity_spawn_packets(stream, compression, item, item_pid)?;
            }
        }
    }

    // Hook A: wall-clock timer driving item entity age ticks at ~20 Hz (50 ms per tick).
    // Java: ItemEntity.tick() — called once per server tick, ~50 ms.
    let mut last_item_tick = Instant::now();
    let mut last_player_tick = Instant::now();
    let mut play_tick_count = 0_u64;
    let mut live_fluid_ticks = LiveFluidTicks::new();
    {
        let center =
            chunk_cache.get_or_load(current_chunk_x, current_chunk_z, world_root, world_seed);
        unpack_chunk_fluid_ticks(&mut live_fluid_ticks, play_tick_count as i64, &center);
    }
    const ITEM_TICK_INTERVAL: Duration = Duration::from_millis(50);
    loop {
        if last_keep_alive.elapsed() >= PLAY_KEEP_ALIVE_INTERVAL {
            keep_alive_id = keep_alive_id.wrapping_add(1);
            write_framed_packet_with_compression(
                stream,
                compression,
                CLIENTBOUND_KEEP_ALIVE_PACKET_ID,
                |payload| payload.write_all(&keep_alive_id.to_be_bytes()),
            )?;
            last_keep_alive = Instant::now();
        }

        // Time heartbeat: empty clock map, just the current game_time.
        // Java: MinecraftServer.forceGameTimeSynchronization() every 20 ticks (~1 second)
        if last_time_sync.elapsed() >= TIME_SYNC_INTERVAL {
            let game_time = clock.lock().unwrap().heartbeat_game_time();
            write_framed_packet_with_compression(
                stream,
                compression,
                CLIENTBOUND_SET_TIME_PACKET_ID,
                |payload| {
                    ClientboundSetTimePacket {
                        game_time,
                        clock_updates: BTreeMap::new(),
                    }
                    .write(payload)
                },
            )?;
            last_time_sync = Instant::now();
        }

        // Hook A: Item entity age tick — ~20 Hz wall-clock.
        // Mirrors ItemEntity.tick(): apply drag, decrement pickupDelay, increment age,
        // expire at LIFETIME, and merge nearby same-type stacks.
        // Java: ServerLevel.tick() → entity.tick() → mergeWithNeighbours() for every ItemEntity.
        if last_item_tick.elapsed() >= ITEM_TICK_INTERVAL {
            last_item_tick = Instant::now();
            let result = {
                let mut items = world_items.lock().unwrap();
                item_entity::tick(&mut items.entities)
            };
            if !result.removed.is_empty() {
                write_framed_packet_with_compression(
                    stream,
                    compression,
                    CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID,
                    |p| {
                        write_var_i32(p, result.removed.len() as i32)?;
                        for id in &result.removed {
                            write_var_i32(p, *id)?;
                        }
                        Ok(())
                    },
                )?;
            }
            // Notify the client of any count changes caused by stack merges.
            // Note: count-update SET_ENTITY_DATA is NOT bundled — bundles are only needed
            // for the initial ADD_ENTITY + SET_ENTITY_DATA spawn pair.
            for (entity_id, item_name, new_count) in &result.count_updates {
                if let Some(item_pid) = item_protocol_id(item_name) {
                    write_framed_packet_with_compression(
                        stream,
                        compression,
                        CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID,
                        |p| {
                            write_var_i32(p, *entity_id)?;
                            p.write_all(&[8u8])?; // index 8: ItemEntity.DATA_ITEM
                            write_var_i32(p, 7)?; // serializer 7: ITEM_STACK
                            write_var_i32(p, *new_count)?;
                            write_var_i32(p, item_pid)?;
                            write_var_i32(p, 0)?; // component add count
                            write_var_i32(p, 0)?; // component remove count
                            p.write_all(&[0xFFu8]) // end of metadata
                        },
                    )?;
                }
            }
        }

        // Java: ServerPlayer.doTick() calls FoodData.tick(this) every server
        // tick, independent of inbound movement/interaction packets. Entity
        // base ticking updates fluid contact and air supply on the same tick.
        if last_player_tick.elapsed() >= SERVER_TICK_DURATION {
            last_player_tick = Instant::now();
            play_tick_count = play_tick_count.wrapping_add(1);
            process_live_fluid_ticks(
                stream,
                compression,
                &mut live_fluid_ticks,
                play_tick_count as i64,
                &world_layout,
                world_seed,
                chunk_cache,
            )?;
            let fluid_state =
                detect_play_session_fluid_state(&play_state, world_root, world_seed, chunk_cache);
            let water_update = tick_play_session_water(&mut play_state, fluid_state);
            if water_update.air_changed {
                write_play_state_air_supply_packet(stream, compression, &play_state)?;
            }
            if water_update.motion_changed {
                write_play_state_motion_packet(stream, compression, &play_state)?;
            }
            if tick_play_session_food(
                &mut play_state,
                food_difficulty_from_properties(properties),
                true,
                play_tick_count,
            ) || water_update.health_changed
            {
                write_play_state_health_packet(stream, compression, &play_state)?;
            }

            // Per-tick chunk send drain (Java mirror:
            // MinecraftServer.tickChildren → chunkSender.sendNextChunks).
            // Sits at the end of the player tick so fluid/entity ticking
            // sees the same chunk snapshot as the chunks being flushed.
            let drained = drain_chunk_sender(
                stream,
                compression,
                &mut chunk_sender,
                chunk_pipeline,
                ChunkPos {
                    x: current_chunk_x,
                    z: current_chunk_z,
                },
                Some((&mut live_fluid_ticks, play_tick_count as i64)),
            )?;
            chunk_pipeline_stats.sent_total = chunk_pipeline_stats
                .sent_total
                .saturating_add(drained as u64);
            maybe_log_chunk_pipeline_stats(
                &mut chunk_pipeline_stats,
                &chunk_sender,
                chunk_pipeline,
                play_tick_count,
            );
        }

        // Detect weather level changes and broadcast to client.
        // Java: ServerLevel.advanceWeatherCycle() — RainLevelChange/ThunderLevelChange
        {
            let (cur_rain, cur_thunder) = {
                let wc = weather.lock().unwrap();
                (wc.rain_level, wc.thunder_level)
            };
            if (cur_rain - last_sent_rain_level).abs() > f32::EPSILON {
                write_game_event(stream, compression, 7, cur_rain)?;
                // Also send StopRaining(2) or StartRaining(1) on boundary crossings.
                // Java: WeatherGameEvent::StopRaining/StartRaining at rain_level 0.2 threshold
                if last_sent_rain_level > 0.2 && cur_rain <= 0.2 {
                    write_game_event(stream, compression, 2, 0.0)?;
                } else if last_sent_rain_level <= 0.2 && cur_rain > 0.2 {
                    write_game_event(stream, compression, 1, 0.0)?;
                }
                last_sent_rain_level = cur_rain;
            }
            if (cur_thunder - last_sent_thunder_level).abs() > f32::EPSILON {
                write_game_event(stream, compression, 8, cur_thunder)?;
                last_sent_thunder_level = cur_thunder;
            }
        }

        match read_packet_with_compression(stream, compression) {
            Ok(packet) => {
                if let PacketRateDecision::Kick { reason } =
                    rate_limiter.record_packet(Instant::now())
                {
                    // Java: InventoryMenu.removed() clears the crafting grid and returns
                    // items to inventory before the player state is persisted.
                    play_state.inventory_menu.clear_crafting_to_inventory();
                    let _ =
                        save_play_session_state(world_root, &finished.profile.uuid, &play_state);
                    save_world_item_entities(world_root, &world_items.lock().unwrap());
                    // Flush any in-memory block changes (player edits,
                    // fluid spreads) that haven't yet hit the periodic
                    // 30 s flush window — disconnect must not lose work.
                    chunk_cache.flush_dirty(world_root);
                    write_framed_packet_with_compression(
                        stream,
                        compression,
                        CLIENTBOUND_DISCONNECT_PACKET_ID,
                        |payload| {
                            ClientboundDisconnectPacket {
                                reason: ComponentJson(format!("{{\"translate\":\"{reason}\"}}")),
                            }
                            .write(payload)
                        },
                    )?;
                    return Ok(());
                }
                let mut input = Cursor::new(packet);
                let packet_id = read_var_i32(&mut input)?;
                let session_update =
                    update_play_session_state(packet_id, &mut input, &mut play_state)?;
                if session_update.health_changed {
                    write_play_state_health_packet(stream, compression, &play_state)?;
                }
                if session_update.respawn_requested {
                    handle_play_respawn_request(
                        stream,
                        compression,
                        &mut play_state,
                        properties,
                        world_root,
                        world_seed,
                    )?;
                    current_chunk_x = chunk_coordinate(play_state.x);
                    current_chunk_z = chunk_coordinate(play_state.z);
                    loaded_chunks =
                        chunk_window(current_chunk_x, current_chunk_z, chunk_batch_radius);
                    // Re-seed the per-session sender for the new spawn location.
                    // Pending chunks from before the respawn no longer make
                    // sense (different center, different visible window).
                    chunk_sender = PlayerChunkSender::new(false);
                    seed_chunk_window(
                        &mut chunk_sender,
                        chunk_pipeline,
                        current_chunk_x,
                        current_chunk_z,
                        chunk_batch_radius,
                    );
                    let _ =
                        save_play_session_state(world_root, &finished.profile.uuid, &play_state);
                    continue;
                }
                if session_update.position_changed {
                    let next_chunk_x = chunk_coordinate(play_state.x);
                    let next_chunk_z = chunk_coordinate(play_state.z);
                    if next_chunk_x != current_chunk_x || next_chunk_z != current_chunk_z {
                        current_chunk_x = next_chunk_x;
                        current_chunk_z = next_chunk_z;
                        // Diff old/new visible windows: forget chunks
                        // leaving the window (or just drop them from
                        // pending if they had not been flushed yet), and
                        // enqueue chunks entering it. Java mirror:
                        // ChunkMap.applyChunkTrackingView when the player's
                        // tracked chunk position changes. The actual chunk
                        // payloads are flushed by the next-tick
                        // drain_chunk_sender; this path never blocks on
                        // worldgen.
                        apply_chunk_movement(
                            stream,
                            compression,
                            &mut chunk_sender,
                            chunk_pipeline,
                            &mut loaded_chunks,
                            current_chunk_x,
                            current_chunk_z,
                            chunk_batch_radius,
                            world_root,
                            world_seed,
                        )?;
                    }
                    // Hook B: Pickup check — mirrors Player.aiStep() proximity sweep.
                    // Spectators cannot pick up items.
                    // Java: Player.aiStep() — inflate AABB, iterate nearby entities, call playerTouch.
                    if play_state.game_mode != GameMode::Spectator {
                        process_item_pickups(
                            stream,
                            compression,
                            &mut play_state,
                            &finished.profile.uuid,
                            world_items,
                        )?;
                    }
                    continue;
                }
                if packet_id == SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID {
                    write_command_suggestions_response(stream, compression, &mut input)?;
                    continue;
                }
                if packet_id == SERVERBOUND_CHAT_PACKET_ID {
                    handle_chat_packet(stream, compression, &mut input, &finished.profile)?;
                    continue;
                }
                if packet_id == SERVERBOUND_CHAT_COMMAND_PACKET_ID
                    || packet_id == SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID
                {
                    handle_chat_command_packet(
                        stream,
                        compression,
                        &mut input,
                        packet_id == SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID,
                        &finished.profile,
                        &mut play_state,
                        properties,
                        player_access,
                        world_seed,
                    )?;
                    continue;
                }
                if packet_id == SERVERBOUND_USE_ITEM_ON_PACKET_ID {
                    let packet = ServerboundUseItemOnPacket::read(&mut input)?;
                    handle_use_item_on(
                        stream,
                        compression,
                        &mut play_state,
                        &world_layout,
                        world_seed,
                        chunk_cache,
                        &mut live_fluid_ticks,
                        play_tick_count as i64,
                        &packet,
                    )?;
                    continue;
                }
                if packet_id == SERVERBOUND_PLAYER_ACTION_PACKET_ID {
                    let action = read_var_i32(&mut input)?;
                    let mut pos_bytes = [0u8; 8];
                    input.read_exact(&mut pos_bytes)?;
                    let packed_pos = i64::from_be_bytes(pos_bytes);
                    let mut direction_byte = [0u8; 1];
                    input.read_exact(&mut direction_byte)?;
                    let sequence = read_var_i32(&mut input)?;
                    let (dbx, dby, dbz) = unpack_block_position(packed_pos);
                    crate::log::log_debug(&format!(
                        "player_action action={action} pos=({dbx},{dby},{dbz}) mode={:?}",
                        play_state.game_mode
                    ));
                    // Java ServerPlayerGameMode: START_DESTROY_BLOCK with getDestroyProgress >= 1.0
                    // (i.e. destroy_time == 0) → "insta mine" — break immediately, same as creative.
                    if action == 0 {
                        let chunk_pos_dbg = ChunkPos {
                            x: dbx.div_euclid(16),
                            z: dbz.div_euclid(16),
                        };
                        let actual_block =
                            read_block_at(&world_layout, world_seed, chunk_pos_dbg, dbx, dby, dbz);
                        let destroy_time = actual_block
                            .as_deref()
                            .and_then(|name| representative_state_definition(name))
                            .map(|def| def.physical.destroy_time);
                        crate::log::log_debug(&format!("instabreak check: actual_block={actual_block:?} destroy_time={destroy_time:?}"));
                    }
                    let is_instabreak =
                        action == 0 && play_state.game_mode != GameMode::Creative && {
                            let chunk_pos_ib = ChunkPos {
                                x: dbx.div_euclid(16),
                                z: dbz.div_euclid(16),
                            };
                            read_block_at(&world_layout, world_seed, chunk_pos_ib, dbx, dby, dbz)
                                .as_deref()
                                .and_then(|name| representative_state_definition(name))
                                .map(|def| def.physical.destroy_time == 0.0)
                                .unwrap_or(false)
                        };
                    let should_break = action == 2
                        || (action == 0 && play_state.game_mode == GameMode::Creative)
                        || is_instabreak;
                    if should_break {
                        // Packet ordering rationale:
                        //
                        // Java defers BlockChangedAck to the start of the next server tick
                        // (~50 ms later via ServerGamePacketListenerImpl.ackBlockChangesUpTo).
                        // In that window the entity is already spawned, physics-ticked, and
                        // rendering on the client.  Any block-prediction rollback triggered by
                        // the delayed ack therefore never touches the stable entity.
                        //
                        // Our server is synchronous — all packets go out in one TCP write.
                        // Testing confirms that sending BlockChangedAck AFTER the entity (Java's
                        // final wire order) causes the client to process the ack and AddEntity in
                        // the same packet loop, triggering prediction rollback while the entity
                        // has just been registered but hasn't been physics-ticked yet — the
                        // rollback culls it (always invisible).
                        //
                        // Sending BlockChangedAck FIRST lets the client commit its block-
                        // prediction state before AddEntity arrives, so the entity spawns into
                        // confirmed-AIR and renders correctly.
                        if crate::log::global_level() >= crate::log::LogLevel::Trace {
                            crate::log::log_trace(&format!(
                                "block break seq={sequence} pos=({dbx},{dby},{dbz}) action={action} game_mode={:?}",
                                play_state.game_mode
                            ));
                            crate::log::log_trace(&format!(
                                "sending BLOCK_CHANGED_ACK seq={sequence}"
                            ));
                        }
                        write_framed_packet_with_compression(
                            stream,
                            compression,
                            CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID,
                            |p| write_var_i32(p, sequence),
                        )?;
                        if crate::log::global_level() >= crate::log::LogLevel::Trace {
                            crate::log::log_trace(&format!(
                                "sending BLOCK_UPDATE pos=({dbx},{dby},{dbz}) new_state=AIR"
                            ));
                        }
                        write_framed_packet_with_compression(
                            stream,
                            compression,
                            CLIENTBOUND_BLOCK_UPDATE_PACKET_ID,
                            |p| {
                                p.write_all(&packed_pos.to_be_bytes())?;
                                write_var_i32(p, AIR_BLOCK_STATE_ID)
                            },
                        )?;
                        let (bx, by, bz) = unpack_block_position(packed_pos);
                        // Java mirror: ServerLevel.removeBlock → LevelChunk.setBlockState
                        // — mutates the in-memory chunk and marks it
                        // unsaved. Persistence happens later via the
                        // periodic flush thread; no per-break disk I/O.
                        let block_name = chunk_cache.set_block(
                            world_root,
                            world_seed,
                            crate::block_update::BlockPos {
                                x: bx,
                                y: by,
                                z: bz,
                            },
                            "minecraft:air",
                        );
                        schedule_neighbor_fluids(
                            &mut live_fluid_ticks,
                            play_tick_count as i64,
                            &world_layout,
                            world_seed,
                            crate::block_update::BlockPos {
                                x: bx,
                                y: by,
                                z: bz,
                            },
                        );
                        crate::log::log_debug(&format!(
                            "block break at ({bx},{by},{bz}) block={block_name:?} game_mode={:?}",
                            play_state.game_mode
                        ));
                        if play_state.game_mode != GameMode::Creative {
                            let loot_seed = (bx as u64).wrapping_mul(0x9E37_79B9)
                                ^ (by as u64).wrapping_mul(0x6C62_272E)
                                ^ (bz as u64).wrapping_mul(0x517C_C1B7);
                            let drops = block_name
                                .as_deref()
                                .map(|n| evaluate_block_loot(n, loot_seed))
                                .unwrap_or_default();
                            let drop_x = bx as f64 + 0.5;
                            let drop_y = by as f64 + 0.5;
                            let drop_z = bz as f64 + 0.5;
                            for (item_name, count) in drops {
                                let Some(item_pid) = item_protocol_id(item_name) else {
                                    continue;
                                };
                                let eid = world_items.lock().unwrap().alloc_entity_id();
                                // Java: ItemEntity constructor sets initial velocity
                                // (random*0.2-0.1, 0.2, random*0.2-0.1) — the y=0.2 upward
                                // component produces the characteristic item "pop" animation
                                // and ensures the entity is visible on spawn.
                                let vel_x = pseudo_rand_f32(eid, 0) as f64 * 0.2 - 0.1;
                                let vel_y = 0.2_f64;
                                let vel_z = pseudo_rand_f32(eid, 1) as f64 * 0.2 - 0.1;
                                let item = DroppedItem {
                                    entity_id: eid,
                                    item: item_name,
                                    count,
                                    x: drop_x,
                                    y: drop_y,
                                    z: drop_z,
                                    vel_x,
                                    vel_y,
                                    vel_z,
                                    pickup_delay: DEFAULT_PICKUP_DELAY,
                                    age: 0,
                                    target_uuid: None,
                                };
                                write_item_entity_spawn_packets(
                                    stream,
                                    compression,
                                    &item,
                                    item_pid,
                                )?;
                                world_items.lock().unwrap().entities.push(item);
                            }
                        }
                    }
                    // Java: ServerboundPlayerActionPacket.Action.DROP_ALL_ITEMS = 3,
                    //        ServerboundPlayerActionPacket.Action.DROP_ITEM = 4.
                    if action == 3 || action == 4 {
                        handle_drop_item(
                            stream,
                            compression,
                            &mut play_state,
                            world_items,
                            action == 3,
                        )?;
                    }
                    continue;
                }
                if packet_id == SERVERBOUND_CONTAINER_CLICK_PACKET_ID {
                    // Only handle player inventory (container_id 0) for now.
                    // Java: ServerGamePacketListenerImpl.handleContainerClick()
                    if let Ok(click) = ServerboundContainerClickPacket::read(&mut input) {
                        if click.container_id == 0 {
                            let instructions = handle_container_click(
                                &click,
                                &mut play_state.container_state_id,
                                &mut play_state.inventory_menu,
                                &mut play_state.carried_item,
                            );
                            for instruction in instructions {
                                match instruction {
                                    PlayInstruction::ContainerSetSlot(pkt) => {
                                        write_framed_packet_with_compression(
                                            stream,
                                            compression,
                                            CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID,
                                            |p| pkt.write(p),
                                        )?;
                                    }
                                    PlayInstruction::SetCursorItem(pkt) => {
                                        write_framed_packet_with_compression(
                                            stream,
                                            compression,
                                            CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID,
                                            |p| pkt.write(p),
                                        )?;
                                    }
                                    PlayInstruction::RecipesUnlocked(ids) => {
                                        if let Some(pkt) =
                                            build_recipe_book_add(&ids, recipe_manager.recipe_map())
                                        {
                                            write_framed_packet_with_compression(
                                                stream,
                                                compression,
                                                CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID,
                                                |p| pkt.write(p),
                                            )?;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    continue;
                }
                if packet_id == SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID {
                    let packet = ServerboundSetCreativeModeSlotPacket::read(&mut input)?;
                    if super::player_creative_packets::apply_set_creative_mode_slot_packet(
                        &mut play_state,
                        packet,
                    ) {
                        write_inventory_menu_full_sync(stream, compression, &play_state)?;
                    }
                    continue;
                }
                if packet_id == SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS_PACKET_ID {
                    let packet = ServerboundRecipeBookChangeSettingsPacket::read(&mut input)?;
                    apply_recipe_book_settings_packet(&mut play_state, packet);
                    continue;
                }
                if packet_id == SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE_PACKET_ID {
                    let packet = ServerboundRecipeBookSeenRecipePacket::read(&mut input)?;
                    apply_recipe_book_seen_recipe_packet(
                        &mut play_state,
                        packet,
                        recipe_manager.recipe_map(),
                    );
                    continue;
                }
                if packet_id == SERVERBOUND_PLACE_RECIPE_PACKET_ID {
                    let packet = ServerboundPlaceRecipePacket::read(&mut input)?;
                    if packet.container_id == 0 {
                        if apply_place_recipe_packet(
                            &mut play_state,
                            packet,
                            recipe_manager.recipe_map(),
                        ) {
                            write_inventory_menu_full_sync(stream, compression, &play_state)?;
                        }
                    }
                    continue;
                }
                if packet_id == SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID {
                    // Java: ServerGamePacketListenerImpl.handleChunkBatchReceived
                    // → PlayerChunkSender.onChunkBatchReceivedByClient. The
                    // payload is a single f32: the client's measured desired
                    // chunks-per-tick. The sender uses it both to clamp pacing
                    // and to lift the unacked-batches gate from 1 → 10.
                    handle_chunk_batch_received_packet(&mut input, &mut chunk_sender)?;
                    continue;
                }
                if play_packet_is_handled_after_state_update(packet_id) {
                    continue;
                }
                play_state.inventory_menu.clear_crafting_to_inventory();
                let _ = save_play_session_state(world_root, &finished.profile.uuid, &play_state);
                save_world_item_entities(world_root, &world_items.lock().unwrap());
                chunk_cache.flush_dirty(world_root);
                write_framed_packet_with_compression(
                    stream,
                    compression,
                    CLIENTBOUND_DISCONNECT_PACKET_ID,
                    |payload| {
                        ClientboundDisconnectPacket {
                            reason: ComponentJson(format!(
                                "{{\"text\":\"unexpected play packet {packet_id}\"}}"
                            )),
                        }
                        .write(payload)
                    },
                )?;
                return Ok(());
            }
            Err(err)
                if matches!(
                    err.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) => {}
            Err(err)
                if matches!(
                    err.kind(),
                    io::ErrorKind::UnexpectedEof | io::ErrorKind::ConnectionReset
                ) =>
            {
                play_state.inventory_menu.clear_crafting_to_inventory();
                let _ = save_play_session_state(world_root, &finished.profile.uuid, &play_state);
                save_world_item_entities(world_root, &world_items.lock().unwrap());
                chunk_cache.flush_dirty(world_root);
                return Ok(());
            }
            Err(err) => return Err(err),
        }
    }
}
