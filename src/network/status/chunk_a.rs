use super::*;

const RESOURCE_USAGE_LOG_INTERVAL_TICKS: u64 = 100;

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

struct StatusServerRuntime {
    address: String,
    listener: TcpListener,
    favicon: Option<String>,
    active_logins: ActiveLoginRegistry,
    world_root: Arc<PathBuf>,
    chunk_cache: GeneratedChunkCache,
    chunk_pipeline: ChunkPipeline,
    player_access: Arc<Mutex<PlayerAccess>>,
    clock: Arc<Mutex<ServerClockManager>>,
    weather: Arc<Mutex<WeatherCycle>>,
    recipe_manager: Arc<RecipeManagerModel>,
    world_items: Arc<Mutex<WorldItemEntities>>,
    world_mobs: Arc<Mutex<LiveMobStore>>,
    max_tick_time: Duration,
}

#[derive(Clone, Copy)]
struct ConnectionSharedContext<'a> {
    properties: &'a ServerProperties,
    favicon: Option<&'a str>,
    active_logins: &'a ActiveLoginRegistry,
    chunk_cache: &'a GeneratedChunkCache,
    chunk_pipeline: &'a ChunkPipeline,
    player_access: &'a Arc<Mutex<PlayerAccess>>,
    world_root: &'a Path,
    world_seed: i64,
    clock: &'a Arc<Mutex<ServerClockManager>>,
    weather: &'a Arc<Mutex<WeatherCycle>>,
    recipe_manager: &'a RecipeManagerModel,
    world_items: &'a Arc<Mutex<WorldItemEntities>>,
    world_mobs: &'a Arc<Mutex<LiveMobStore>>,
}

struct StatusConnectionContext<'a> {
    shared: ConnectionSharedContext<'a>,
    remote_address: &'a str,
    remote_ip: &'a str,
}

struct LoginConnectionContext<'a> {
    shared: ConnectionSharedContext<'a>,
    remote_address: &'a str,
    remote_ip: &'a str,
    login_host_ip: Option<String>,
    rate_limiter: &'a mut PacketRateLimiter,
}

struct PlayConnectionContext<'a> {
    shared: ConnectionSharedContext<'a>,
    remote_address: &'a str,
    rate_limiter: &'a mut PacketRateLimiter,
    /// The player's registry guard, so play-phase `ClientInformation` updates can
    /// propagate the listing preference to the status player sample.
    active_login: &'a ActiveLoginGuard,
}

struct JoinedPlaySessionStart {
    play_state: PlaySessionState,
    current_chunk_x: i32,
    current_chunk_z: i32,
    chunk_batch_radius: i32,
    loaded_chunks: BTreeSet<(i32, i32)>,
    chunk_sender: PlayerChunkSender,
    chunk_pipeline_stats: ChunkPipelineSessionStats,
    /// Keepalive challenge/timeout tracker (Java `ServerCommonPacketListenerImpl`
    /// keepAlive fields), driven off `keep_alive_epoch` for millisecond timestamps.
    keep_alive: KeepAliveState,
    keep_alive_epoch: Instant,
    last_sent_rain_level: f32,
    last_sent_thunder_level: f32,
    last_time_sync: Instant,
    world_layout: WorldLayout,
    last_item_tick: Instant,
    last_player_tick: Instant,
    play_tick_count: u64,
    live_fluid_ticks: LiveFluidTicks,
    live_block_ticks: LiveBlockTicks,
}

const ITEM_TICK_INTERVAL: Duration = Duration::from_millis(50);

/// Maps the play-session [`GameMode`] to the command model's game mode (and back
/// via [`play_game_mode`]). Used when seeding/applying `/gamemode` command state.
pub(super) fn command_game_mode(game_mode: GameMode) -> crate::command::GameMode {
    match game_mode {
        GameMode::Survival => crate::command::GameMode::Survival,
        GameMode::Creative => crate::command::GameMode::Creative,
        GameMode::Adventure => crate::command::GameMode::Adventure,
        GameMode::Spectator => crate::command::GameMode::Spectator,
    }
}

pub(super) fn play_game_mode(game_mode: crate::command::GameMode) -> GameMode {
    match game_mode {
        crate::command::GameMode::Survival => GameMode::Survival,
        crate::command::GameMode::Creative => GameMode::Creative,
        crate::command::GameMode::Adventure => GameMode::Adventure,
        crate::command::GameMode::Spectator => GameMode::Spectator,
    }
}

/// Handshake `ClientIntent` ids, 1:1 with Java `ClientIntent` (`STATUS_ID`/
/// `LOGIN_ID`/`TRANSFER_ID`).
const INTENTION_STATUS: i32 = 1;
const INTENTION_LOGIN: i32 = 2;
const INTENTION_TRANSFER: i32 = 3;

pub(super) fn lock_status_mutex<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

pub fn run_status_server(
    bind_ip: &str,
    port: u16,
    properties: &ServerProperties,
    world_root: &Path,
    world_seed: i64,
    console_input: &Receiver<ConsoleInput>,
    active_logins: ActiveLoginRegistry,
) -> Result<(), String> {
    let runtime = StatusServerRuntime::new(
        bind_ip,
        port,
        properties,
        world_root,
        world_seed,
        active_logins,
    )?;
    runtime.start_tick_thread();
    println!("Status listener bound to {}", runtime.address);
    run_status_accept_loop(runtime, properties, world_seed, console_input);
    Ok(())
}

impl StatusServerRuntime {
    fn new(
        bind_ip: &str,
        port: u16,
        properties: &ServerProperties,
        world_root: &Path,
        world_seed: i64,
        active_logins: ActiveLoginRegistry,
    ) -> Result<Self, String> {
        let address = format!("{bind_ip}:{port}");
        let listener = TcpListener::bind(&address)
            .map_err(|err| format!("Failed to bind status listener on {address}: {err}"))?;
        listener
            .set_nonblocking(true)
            .map_err(|err| format!("Failed to configure status listener on {address}: {err}"))?;
        // Java MinecraftServer.loadStatusIcon: prefer server-icon.png, fall back
        // to the world's icon.png, and tolerate a bad icon (log + no favicon)
        // rather than failing startup. The active-login registry is shared with
        // the GS4 query listener so both report the same live player set.
        let favicon = resolve_status_icon(world_root);
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
        // default is 5 min / `VibecraftDefault.WORLD_AUTOSAVE_INTERVAL`); a
        // shorter window keeps the disconnect-vs-save race tight without
        // making the writes themselves any more expensive.
        spawn_chunk_flush_thread(
            chunk_cache.clone(),
            Arc::clone(&world_root),
            Duration::from_secs(30),
            properties.sync_chunk_writes,
            RegionCompression::from_property_value(&properties.region_file_compression),
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
        let world_mobs: Arc<Mutex<LiveMobStore>> = Arc::new(Mutex::new(LiveMobStore::default()));

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
        let max_tick_time = Duration::from_millis(properties.max_tick_time);
        Ok(Self {
            address,
            listener,
            favicon,
            active_logins,
            world_root,
            chunk_cache,
            chunk_pipeline,
            player_access,
            clock,
            weather,
            recipe_manager,
            world_items,
            world_mobs,
            max_tick_time,
        })
    }

    fn start_tick_thread(&self) {
        let clock_t = Arc::clone(&self.clock);
        let weather_t = Arc::clone(&self.weather);
        let world_root_t = Arc::clone(&self.world_root);
        let world_items_t = Arc::clone(&self.world_items);
        let max_tick_time = self.max_tick_time;
        thread::spawn(move || {
            let mut scheduled = ScheduledTimeChanges::default();
            let mut next_tick = Instant::now() + SERVER_TICK_DURATION;
            let mut tick_count: u64 = 0;
            let mut resource_usage = ResourceUsageSampler::new();
            loop {
                let now = Instant::now();
                if now < next_tick {
                    thread::sleep(next_tick - now);
                }
                next_tick += SERVER_TICK_DURATION;
                tick_count += 1;
                let tick_start = Instant::now();

                // advance_time=true: no per-world gamerule access yet; always advance.
                lock_status_mutex(&clock_t).tick(true, &mut scheduled);

                // Advance weather. can_have_weather=true for overworld.
                lock_status_mutex(&weather_t).advance(true, true, sample_weather_durations());

                if tick_count.is_multiple_of(RESOURCE_USAGE_LOG_INTERVAL_TICKS) {
                    resource_usage.log_current_usage(tick_count);
                }

                // Persist every ~5 minutes.
                // Java: MinecraftServer.saveEverything() — entities flushed via EntityStorage.
                if tick_count.is_multiple_of(PERSISTENCE_INTERVAL_TICKS) {
                    save_server_clock_state(&world_root_t, &lock_status_mutex(&clock_t));
                    save_server_weather_state(&world_root_t, &lock_status_mutex(&weather_t));
                    save_world_item_entities(&world_root_t, &lock_status_mutex(&world_items_t));
                }

                // Watchdog: warn if tick exceeded max-tick-time.
                // Java ServerWatchdog crashes the server; we log a warning
                // (crash behavior requires a dedicated watchdog thread).
                if !max_tick_time.is_zero() {
                    let tick_elapsed = tick_start.elapsed();
                    if tick_elapsed > max_tick_time {
                        log_info(&format!(
                            "Server tick #{tick_count} took {}ms (max-tick-time={}ms)",
                            tick_elapsed.as_millis(),
                            max_tick_time.as_millis()
                        ));
                    }
                }
            }
        });
    }

    fn save_shared_state(&self) {
        save_server_clock_state(&self.world_root, &lock_status_mutex(&self.clock));
        save_server_weather_state(&self.world_root, &lock_status_mutex(&self.weather));
        save_world_item_entities(&self.world_root, &lock_status_mutex(&self.world_items));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CpuUsageSample {
    process_jiffies: u64,
    total_jiffies: u64,
}

#[derive(Debug)]
struct ResourceUsageSampler {
    previous_cpu_sample: Option<CpuUsageSample>,
    logical_cpus: u64,
}

impl ResourceUsageSampler {
    fn new() -> Self {
        Self {
            previous_cpu_sample: read_cpu_usage_sample(),
            logical_cpus: std::thread::available_parallelism()
                .map(|count| count.get() as u64)
                .unwrap_or(1),
        }
    }

    fn log_current_usage(&mut self, tick_count: u64) {
        let memory = current_resident_memory_kib()
            .map(|kib| format!("{} MiB", kib / 1024))
            .unwrap_or_else(|| "unavailable".to_string());
        let cpu = self
            .current_cpu_percent()
            .map(|percent| format!("{percent:.1}%"))
            .unwrap_or_else(|| "unavailable".to_string());
        log_info(&format!(
            "resource usage at tick {tick_count}: memory={memory}, cpu={cpu}"
        ));
    }

    fn current_cpu_percent(&mut self) -> Option<f64> {
        let current = read_cpu_usage_sample()?;
        let previous = self.previous_cpu_sample.replace(current)?;
        let process_delta = current
            .process_jiffies
            .checked_sub(previous.process_jiffies)?;
        let total_delta = current.total_jiffies.checked_sub(previous.total_jiffies)?;
        if total_delta == 0 {
            return None;
        }
        let cpu_fraction = process_delta as f64 / total_delta as f64;
        Some(cpu_fraction * self.logical_cpus as f64 * 100.0)
    }
}

fn read_cpu_usage_sample() -> Option<CpuUsageSample> {
    Some(CpuUsageSample {
        process_jiffies: read_process_cpu_jiffies(&fs::read_to_string("/proc/self/stat").ok()?)?,
        total_jiffies: read_total_cpu_jiffies(&fs::read_to_string("/proc/stat").ok()?)?,
    })
}

fn read_process_cpu_jiffies(stat: &str) -> Option<u64> {
    let end_of_comm = stat.rfind(") ")?;
    let fields_after_comm = stat.get(end_of_comm + 2..)?.split_whitespace();
    let fields = fields_after_comm.collect::<Vec<_>>();
    let user_time = fields.get(11)?.parse::<u64>().ok()?;
    let system_time = fields.get(12)?.parse::<u64>().ok()?;
    user_time.checked_add(system_time)
}

fn read_total_cpu_jiffies(stat: &str) -> Option<u64> {
    let cpu_line = stat.lines().find(|line| line.starts_with("cpu "))?;
    cpu_line
        .split_whitespace()
        .skip(1)
        .map(str::parse::<u64>)
        .try_fold(0_u64, |total, value| total.checked_add(value.ok()?))
}

fn current_resident_memory_kib() -> Option<u64> {
    let status = fs::read_to_string("/proc/self/status").ok()?;
    parse_resident_memory_kib(&status)
}

fn parse_resident_memory_kib(status: &str) -> Option<u64> {
    let line = status.lines().find(|line| line.starts_with("VmRSS:"))?;
    line.split_whitespace().nth(1)?.parse::<u64>().ok()
}

fn run_status_accept_loop(
    runtime: StatusServerRuntime,
    properties: &ServerProperties,
    world_seed: i64,
    console_input: &Receiver<ConsoleInput>,
) {
    loop {
        if should_stop(console_input, &runtime.player_access) {
            println!("Status listener stopping");
            runtime.save_shared_state();
            break;
        }
        match runtime.listener.accept() {
            Ok((stream, peer_addr)) => {
                let properties = properties.clone();
                let favicon = runtime.favicon.clone();
                let active_logins = runtime.active_logins.clone();
                let chunk_cache = runtime.chunk_cache.clone();
                let chunk_pipeline = runtime.chunk_pipeline.clone();
                let world_root = Arc::clone(&runtime.world_root);
                let player_access = Arc::clone(&runtime.player_access);
                let clock = Arc::clone(&runtime.clock);
                let weather = Arc::clone(&runtime.weather);
                let recipe_manager = Arc::clone(&runtime.recipe_manager);
                let world_items = Arc::clone(&runtime.world_items);
                let world_mobs = Arc::clone(&runtime.world_mobs);
                let remote_ip = peer_addr.ip().to_string();
                let remote_address = peer_addr.to_string();
                let remote_for_log = loggable_remote_address(properties.log_ips, &remote_address);
                thread::spawn(move || {
                    let context = StatusConnectionContext {
                        shared: ConnectionSharedContext {
                            properties: &properties,
                            favicon: favicon.as_deref(),
                            active_logins: &active_logins,
                            chunk_cache: &chunk_cache,
                            chunk_pipeline: &chunk_pipeline,
                            player_access: &player_access,
                            world_root: &world_root,
                            world_seed,
                            clock: &clock,
                            weather: &weather,
                            recipe_manager: &recipe_manager,
                            world_items: &world_items,
                            world_mobs: &world_mobs,
                        },
                        remote_address: &remote_address,
                        remote_ip: &remote_ip,
                    };
                    if let Err(err) = handle_status_connection(stream, context) {
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

fn handle_status_connection(
    mut stream: TcpStream,
    context: StatusConnectionContext<'_>,
) -> io::Result<()> {
    let StatusConnectionContext {
        shared,
        remote_address,
        remote_ip,
    } = context;
    let properties = shared.properties;
    let favicon = shared.favicon;
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;

    let mut first = [0u8; 1];
    if stream.peek(&mut first)? == 1 && first[0] == 0xFE {
        return handle_legacy_status_tcp_connection(
            &mut stream,
            properties,
            shared.active_logins.online_count(),
        );
    }

    let mut rate_limiter =
        PacketRateLimiter::new(properties.rate_limit_packets_per_second, Instant::now());
    let handshake =
        read_packet_with_rate_limit(&mut stream, CompressionState::disabled(), &mut rate_limiter)?;
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
    // Java ServerHandshakePacketListenerImpl.handleIntention routes by ClientIntent:
    // STATUS=1, LOGIN=2, TRANSFER=3 (`ClientIntent.byId`); any other id throws
    // (→ connection closed). LOGIN and TRANSFER share `beginLogin` (protocol-version
    // check then login), but TRANSFER is first gated on `acceptsTransfers()`
    // (`accepts-transfers`, default false) — when disabled it is rejected with a
    // login-state `multiplayer.disconnect.transfers_disabled` disconnect.
    if next_state == INTENTION_LOGIN || next_state == INTENTION_TRANSFER {
        if next_state == INTENTION_TRANSFER && !properties.accepts_transfers {
            return write_transfers_disabled_disconnect(&mut stream);
        }
        if protocol != PROTOCOL_VERSION {
            return write_login_protocol_mismatch_disconnect(&mut stream, protocol);
        }
        // The `transferred` flag (Java `CommonListenerCookie.transferred`) is
        // connection metadata for the transfer-cookie flow with no observable
        // effect on the offline login/configuration/play handshake, so both
        // intents proceed through the same login path.
        return handle_login_connection(
            &mut stream,
            LoginConnectionContext {
                shared,
                remote_address,
                remote_ip,
                login_host_ip: login_host_ip(&server_address),
                rate_limiter: &mut rate_limiter,
            },
        );
    }
    if next_state != INTENTION_STATUS {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported handshake target state",
        ));
    }
    if !properties.enable_status {
        return Ok(());
    }

    let mut status_requested = false;
    loop {
        let packet = read_packet_with_rate_limit(
            &mut stream,
            CompressionState::disabled(),
            &mut rate_limiter,
        )?;
        let mut input = Cursor::new(packet);
        match read_var_i32(&mut input)? {
            0 => {
                // Java ServerStatusPacketListenerImpl.handleStatusRequest: a second
                // status request disconnects (multiplayer.status.request_handled).
                // The status state has no clientbound disconnect packet, so the
                // connection is simply closed.
                if status_requested {
                    return Ok(());
                }
                status_requested = true;
                let json = status_json(properties, favicon, &shared.active_logins.status_players());
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

/// 1:1 with Java `ServerHandshakePacketListenerImpl.handleIntention` TRANSFER case
/// when `acceptsTransfers()` is false: send a login-state disconnect with the
/// `multiplayer.disconnect.transfers_disabled` reason (no translation args) and
/// close. The outbound protocol is already LOGIN at this point, so the login
/// disconnect packet form is correct.
pub fn write_transfers_disabled_disconnect(stream: &mut TcpStream) -> io::Result<()> {
    write_framed_packet(stream, CLIENTBOUND_LOGIN_DISCONNECT_PACKET_ID, |payload| {
        ClientboundLoginDisconnectPacket {
            reason: crate::network::codec::ComponentJson(
                "{\"translate\":\"multiplayer.disconnect.transfers_disabled\"}".to_string(),
            ),
        }
        .write(payload)
    })
}

pub fn login_compression_threshold(properties: &ServerProperties) -> Option<i32> {
    (properties.network_compression_threshold >= 0)
        .then_some(properties.network_compression_threshold)
}

fn is_rate_limit_disconnect_error(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::PermissionDenied
        && err.to_string() == "disconnect.exceeded_packet_rate"
}

fn write_login_rate_limit_disconnect(
    stream: &mut TcpStream,
    compression: CompressionState,
    reason: &str,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_LOGIN_DISCONNECT_PACKET_ID,
        |payload| {
            ClientboundLoginDisconnectPacket {
                reason: ComponentJson(format!("{{\"translate\":\"{reason}\"}}")),
            }
            .write(payload)
        },
    )
}

fn write_configuration_rate_limit_disconnect(
    stream: &mut TcpStream,
    compression: CompressionState,
    reason: &str,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_DISCONNECT_PACKET_ID,
        |payload| {
            ClientboundDisconnectPacket {
                reason: ComponentJson(format!("{{\"translate\":\"{reason}\"}}")),
            }
            .write(payload)
        },
    )
}

fn wait_for_configuration_packet_or_rate_disconnect(
    stream: &mut TcpStream,
    compression: CompressionState,
    expected_packet_id: i32,
    expected_name: &'static str,
    rate_limiter: &mut PacketRateLimiter,
    active_login: &ActiveLoginGuard,
) -> io::Result<()> {
    match wait_for_configuration_packet_with_rate_limit(
        stream,
        compression,
        expected_packet_id,
        expected_name,
        rate_limiter,
        Some(active_login),
    ) {
        Ok(()) => Ok(()),
        Err(err) if is_rate_limit_disconnect_error(&err) => {
            write_configuration_rate_limit_disconnect(stream, compression, &err.to_string())
        }
        Err(err) => Err(err),
    }
}

fn write_vanilla_feature_flags_packet<W: Write>(payload: &mut W) -> io::Result<()> {
    let vanilla = Identifier::parse("minecraft:vanilla").map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid built-in feature flag identifier: {err}"),
        )
    })?;
    write_var_i32(payload, 1)?;
    write_identifier(payload, &vanilla)
}

struct CompletedLogin {
    finished: ClientboundLoginFinishedPacket,
    compression: CompressionState,
    // Held for the whole play session so the player stays registered in the
    // shared ActiveLoginRegistry until disconnect; dropping it deregisters.
    active_login: ActiveLoginGuard,
}

enum LoginHandshakeOutcome {
    Complete(CompletedLogin),
    Closed,
}

fn complete_login_handshake(
    stream: &mut TcpStream,
    context: &mut LoginConnectionContext<'_>,
) -> io::Result<LoginHandshakeOutcome> {
    let mut login = LoginSession::default();
    let hello = match read_expected_login_hello_packet(stream, context.rate_limiter) {
        Ok(hello) => hello,
        Err(err) if is_rate_limit_disconnect_error(&err) => {
            return write_login_rate_limit_disconnect(
                stream,
                CompressionState::disabled(),
                &err.to_string(),
            )
            .map(|()| LoginHandshakeOutcome::Closed);
        }
        Err(err) => return Err(err),
    };
    let finished = login.accept_offline_hello(hello);
    if let Some(reason) = login_access_disconnect_reason(
        context.shared.properties,
        context.shared.player_access,
        &finished.profile,
        context.remote_ip,
        context.login_host_ip.as_deref(),
    )? {
        write_framed_packet(stream, CLIENTBOUND_LOGIN_DISCONNECT_PACKET_ID, |payload| {
            ClientboundLoginDisconnectPacket {
                reason: crate::network::codec::ComponentJson(format!(
                    "{{\"translate\":\"{reason}\"}}"
                )),
            }
            .write(payload)
        })?;
        return Ok(LoginHandshakeOutcome::Closed);
    }
    let (active_login, replaced_stream) = context.shared.active_logins.register_replacing(
        &finished.profile.uuid,
        &finished.profile.name,
        stream,
    )?;
    if let Some(replaced_stream) = replaced_stream {
        let _ = replaced_stream.shutdown(Shutdown::Both);
    }
    cache_login_profile(context.shared.player_access, &finished.profile)?;
    let compression =
        send_login_success_packets(stream, context.shared.properties, &mut login, &finished)?;
    wait_for_login_acknowledgement(stream, compression, context.rate_limiter, &mut login)?;
    Ok(LoginHandshakeOutcome::Complete(CompletedLogin {
        finished,
        compression,
        active_login,
    }))
}

fn send_login_success_packets(
    stream: &mut TcpStream,
    properties: &ServerProperties,
    login: &mut LoginSession,
    finished: &ClientboundLoginFinishedPacket,
) -> io::Result<CompressionState> {
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
    Ok(compression)
}

fn wait_for_login_acknowledgement(
    stream: &mut TcpStream,
    compression: CompressionState,
    rate_limiter: &mut PacketRateLimiter,
    login: &mut LoginSession,
) -> io::Result<()> {
    let packet = match read_packet_with_rate_limit(stream, compression, rate_limiter) {
        Ok(packet) => packet,
        Err(err) if is_rate_limit_disconnect_error(&err) => {
            return write_login_rate_limit_disconnect(stream, compression, &err.to_string());
        }
        Err(err) => return Err(err),
    };
    let mut input = Cursor::new(packet);
    let packet_id = read_var_i32(&mut input)?;
    if packet_id != SERVERBOUND_LOGIN_ACKNOWLEDGED_PACKET_ID {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected login acknowledgement",
        ));
    }
    login.acknowledge(ServerboundLoginAcknowledgedPacket::read(&mut input)?);
    Ok(())
}

type ConfigurationRegistryWriter = fn(&mut Vec<u8>) -> io::Result<()>;

fn write_configuration_registry_packet(
    stream: &mut TcpStream,
    compression: CompressionState,
    writer: ConfigurationRegistryWriter,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        writer,
    )
}

fn write_configuration_registry_packets(
    stream: &mut TcpStream,
    compression: CompressionState,
) -> io::Result<()> {
    let registry_writers: &[ConfigurationRegistryWriter] = &[
        write_minimal_biome_registry_packet::<Vec<u8>>,
        write_vanilla_chat_type_registry_packet::<Vec<u8>>,
        write_vanilla_trim_pattern_registry_packet::<Vec<u8>>,
        write_minimal_trim_material_registry_packet::<Vec<u8>>,
        write_vanilla_wolf_variant_registry_packet::<Vec<u8>>,
        write_vanilla_wolf_sound_variant_registry_packet::<Vec<u8>>,
        write_vanilla_pig_variant_registry_packet::<Vec<u8>>,
        write_vanilla_pig_sound_variant_registry_packet::<Vec<u8>>,
        write_vanilla_frog_variant_registry_packet::<Vec<u8>>,
        write_vanilla_cat_variant_registry_packet::<Vec<u8>>,
        write_vanilla_cat_sound_variant_registry_packet::<Vec<u8>>,
        write_vanilla_cow_sound_variant_registry_packet::<Vec<u8>>,
        write_vanilla_cow_variant_registry_packet::<Vec<u8>>,
        write_vanilla_chicken_sound_variant_registry_packet::<Vec<u8>>,
        write_vanilla_chicken_variant_registry_packet::<Vec<u8>>,
        write_vanilla_zombie_nautilus_variant_registry_packet::<Vec<u8>>,
        write_vanilla_painting_variant_registry_packet::<Vec<u8>>,
        write_minimal_dimension_type_registry_packet::<Vec<u8>>,
        write_minimal_damage_type_registry_packet::<Vec<u8>>,
        write_vanilla_banner_pattern_registry_packet::<Vec<u8>>,
        write_vanilla_jukebox_song_registry_packet::<Vec<u8>>,
        write_vanilla_instrument_registry_packet::<Vec<u8>>,
        write_world_clock_registry_packet::<Vec<u8>>,
        write_vanilla_timeline_registry_packet::<Vec<u8>>,
    ];
    for writer in registry_writers {
        write_configuration_registry_packet(stream, compression, *writer)?;
    }
    Ok(())
}

fn run_configuration_handshake(
    stream: &mut TcpStream,
    properties: &ServerProperties,
    compression: CompressionState,
    rate_limiter: &mut PacketRateLimiter,
    active_login: &ActiveLoginGuard,
) -> io::Result<()> {
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
        write_vanilla_feature_flags_packet,
    )?;
    write_configuration_registry_packets(stream, compression)?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_UPDATE_TAGS_PACKET_ID,
        write_minimal_update_tags_packet,
    )?;
    run_known_pack_configuration_exchange(
        stream,
        properties,
        compression,
        rate_limiter,
        active_login,
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_FINISH_PACKET_ID,
        |_payload| Ok(()),
    )?;
    wait_for_configuration_packet_or_rate_disconnect(
        stream,
        compression,
        SERVERBOUND_CONFIGURATION_FINISH_PACKET_ID,
        "finish configuration",
        rate_limiter,
        active_login,
    )
}

fn run_known_pack_configuration_exchange(
    stream: &mut TcpStream,
    properties: &ServerProperties,
    compression: CompressionState,
    rate_limiter: &mut PacketRateLimiter,
    active_login: &ActiveLoginGuard,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID,
        write_vanilla_known_packs_packet,
    )?;
    wait_for_configuration_packet_or_rate_disconnect(
        stream,
        compression,
        SERVERBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID,
        "selected known packs",
        rate_limiter,
        active_login,
    )?;
    // Java `addOptionalTasks` selects the code of conduct by the client's locale
    // (`clientInformation.language()`), falling back to `en_us` then the first entry
    // (the fallback chain lives in `load_code_of_conduct_for_language`). The client's
    // `ClientInformation` is sent first thing in configuration, so it has been
    // captured into the registry session by this point.
    if let Some(code_of_conduct) =
        load_code_of_conduct_for_language(properties, &active_login.language())?
    {
        write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_CONFIGURATION_CODE_OF_CONDUCT_PACKET_ID,
            |payload| ClientboundCodeOfConductPacket { code_of_conduct }.write(payload),
        )?;
        wait_for_configuration_packet_or_rate_disconnect(
            stream,
            compression,
            SERVERBOUND_CONFIGURATION_ACCEPT_CODE_OF_CONDUCT_PACKET_ID,
            "code of conduct acceptance",
            rate_limiter,
            active_login,
        )?;
    }
    Ok(())
}

fn initialize_joined_play_session(
    stream: &mut TcpStream,
    compression: CompressionState,
    finished: &ClientboundLoginFinishedPacket,
    shared: ConnectionSharedContext<'_>,
    remote_address: &str,
) -> io::Result<JoinedPlaySessionStart> {
    let play_state = load_play_session_state(
        shared.world_root,
        &finished.profile.uuid,
        shared.properties,
        shared.recipe_manager.recipe_map(),
        shared.world_seed,
    );

    // Snapshot current clock and weather state for the join packet.
    // Java: ServerClockManager.createFullSyncPacket() on player join, ServerLevel.sendLevelInfo()
    let (join_game_time, join_clock_data) = lock_status_mutex(shared.clock).full_sync_data(true);
    let (join_rain_level, join_thunder_level) = {
        let weather = lock_status_mutex(shared.weather);
        (weather.rain_level, weather.thunder_level)
    };

    write_minimal_play_join(
        stream,
        compression,
        MinimalPlayJoinContext {
            properties: shared.properties,
            world_seed: shared.world_seed,
            profile: &finished.profile,
            play_state: &play_state,
            recipe_manager: shared.recipe_manager,
            world_root: shared.world_root,
            clock_game_time: join_game_time,
            clock_data: &join_clock_data,
            rain_level: join_rain_level,
            thunder_level: join_thunder_level,
        },
    )?;
    log_info(&player_login_log_message(
        &finished.profile.name,
        &loggable_remote_address(shared.properties.log_ips, remote_address),
        1,
        play_state.x,
        play_state.y,
        play_state.z,
    ));

    let current_chunk_x = chunk_coordinate(play_state.x);
    let current_chunk_z = chunk_coordinate(play_state.z);
    let chunk_batch_radius = chunk_batch_radius(shared.properties);
    let loaded_chunks = chunk_window(current_chunk_x, current_chunk_z, chunk_batch_radius);
    let mut chunk_sender = PlayerChunkSender::new(false);
    seed_chunk_window(
        &mut chunk_sender,
        shared.chunk_pipeline,
        current_chunk_x,
        current_chunk_z,
        chunk_batch_radius,
    );
    stream.set_read_timeout(Some(SERVER_TICK_DURATION))?;
    send_existing_item_entities(stream, compression, shared.world_items)?;

    let mut live_fluid_ticks = LiveFluidTicks::new();
    let live_block_ticks = LiveBlockTicks::new();
    let play_tick_count = 0_u64;
    let center = shared.chunk_cache.get_or_load(
        current_chunk_x,
        current_chunk_z,
        shared.world_root,
        shared.world_seed,
    );
    unpack_chunk_fluid_ticks(&mut live_fluid_ticks, play_tick_count as i64, &center);

    Ok(JoinedPlaySessionStart {
        play_state,
        current_chunk_x,
        current_chunk_z,
        chunk_batch_radius,
        loaded_chunks,
        chunk_sender,
        chunk_pipeline_stats: ChunkPipelineSessionStats::default(),
        keep_alive: KeepAliveState::new(0, 0),
        keep_alive_epoch: Instant::now(),
        last_sent_rain_level: join_rain_level,
        last_sent_thunder_level: join_thunder_level,
        last_time_sync: Instant::now(),
        world_layout: WorldLayout::new(shared.world_root),
        last_item_tick: Instant::now(),
        last_player_tick: Instant::now(),
        play_tick_count,
        live_fluid_ticks,
        live_block_ticks,
    })
}

fn send_existing_item_entities(
    stream: &mut TcpStream,
    compression: CompressionState,
    world_items: &Arc<Mutex<WorldItemEntities>>,
) -> io::Result<()> {
    // On login: re-send ADD_ENTITY + SET_ENTITY_DATA bundles for every item entity that
    // is already on the ground.  This mirrors Java's ServerEntity.addPairing() called during
    // ChunkMap.updatePlayerMobTypeMap() when a player enters tracking range of an entity.
    // Without this, items dropped before a disconnect are invisible after reconnecting.
    let items = lock_status_mutex(world_items);
    for item in &items.entities {
        if let Some(item_pid) = item_protocol_id(item.item) {
            write_item_entity_spawn_packets(stream, compression, item, item_pid)?;
        }
    }
    Ok(())
}

/// Returns `false` once the keepalive has timed out (the `disconnect.timeout`
/// packet has already been written and the play loop should end the session).
pub(super) fn tick_keep_alive_and_time(
    stream: &mut TcpStream,
    compression: CompressionState,
    clock: &Arc<Mutex<ServerClockManager>>,
    keep_alive: &mut KeepAliveState,
    keep_alive_epoch: Instant,
    last_time_sync: &mut Instant,
) -> io::Result<bool> {
    // 1:1 with Java `ServerCommonPacketListenerImpl.keepConnectionAlive`: every 15 s
    // (`KeepAliveState::VANILLA_INTERVAL_MS`) send a `ClientboundKeepAlivePacket`
    // whose challenge is the current ms; if the previous ping is still unanswered
    // when the next interval fires, disconnect with `disconnect.timeout`. The live
    // socket is never "singleplayer owner" (real connection), so that gate is false.
    let now_ms = keep_alive_epoch.elapsed().as_millis() as u64;
    match keep_alive.tick(now_ms, false) {
        KeepAliveTick::Idle => {}
        KeepAliveTick::Send(packet) => {
            write_framed_packet_with_compression(
                stream,
                compression,
                CLIENTBOUND_KEEP_ALIVE_PACKET_ID,
                |payload| payload.write_all(&packet.id.to_be_bytes()),
            )?;
        }
        KeepAliveTick::Disconnect => {
            write_disconnect_component(stream, compression, "disconnect.timeout")?;
            return Ok(false);
        }
    }

    // Time heartbeat: empty clock map, just the current game_time.
    // Java: MinecraftServer.forceGameTimeSynchronization() every 20 ticks (~1 second)
    if last_time_sync.elapsed() >= TIME_SYNC_INTERVAL {
        let game_time = lock_status_mutex(clock).heartbeat_game_time();
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
        *last_time_sync = Instant::now();
    }
    Ok(true)
}

fn tick_item_entities_for_client(
    stream: &mut TcpStream,
    compression: CompressionState,
    world_items: &Arc<Mutex<WorldItemEntities>>,
    last_item_tick: &mut Instant,
) -> io::Result<()> {
    if last_item_tick.elapsed() < ITEM_TICK_INTERVAL {
        return Ok(());
    }
    *last_item_tick = Instant::now();
    let result = {
        let mut items = lock_status_mutex(world_items);
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
    Ok(())
}

fn broadcast_weather_if_changed(
    stream: &mut TcpStream,
    compression: CompressionState,
    weather: &Arc<Mutex<WeatherCycle>>,
    last_sent_rain_level: &mut f32,
    last_sent_thunder_level: &mut f32,
) -> io::Result<()> {
    let (cur_rain, cur_thunder) = {
        let weather = lock_status_mutex(weather);
        (weather.rain_level, weather.thunder_level)
    };
    if (cur_rain - *last_sent_rain_level).abs() > f32::EPSILON {
        write_game_event(stream, compression, 7, cur_rain)?;
        // Also send StopRaining(2) or StartRaining(1) on boundary crossings.
        // Java: WeatherGameEvent::StopRaining/StartRaining at rain_level 0.2 threshold
        if *last_sent_rain_level > 0.2 && cur_rain <= 0.2 {
            write_game_event(stream, compression, 2, 0.0)?;
        } else if *last_sent_rain_level <= 0.2 && cur_rain > 0.2 {
            write_game_event(stream, compression, 1, 0.0)?;
        }
        *last_sent_rain_level = cur_rain;
    }
    if (cur_thunder - *last_sent_thunder_level).abs() > f32::EPSILON {
        write_game_event(stream, compression, 8, cur_thunder)?;
        *last_sent_thunder_level = cur_thunder;
    }
    Ok(())
}

struct PlayerTickContext<'a, 'b> {
    properties: &'a ServerProperties,
    world_root: &'a Path,
    world_seed: i64,
    chunk_cache: &'a GeneratedChunkCache,
    chunk_pipeline: &'a ChunkPipeline,
    current_chunk_x: i32,
    current_chunk_z: i32,
    chunk_sender: &'b mut PlayerChunkSender,
    chunk_pipeline_stats: &'b mut ChunkPipelineSessionStats,
    live_fluid_ticks: &'b mut LiveFluidTicks,
    live_block_ticks: &'b mut LiveBlockTicks,
    world_items: &'a Arc<Mutex<WorldItemEntities>>,
    world_mobs: &'a Arc<Mutex<LiveMobStore>>,
    loaded_chunks: &'a BTreeSet<(i32, i32)>,
    world_layout: &'b WorldLayout,
    last_player_tick: &'b mut Instant,
    play_tick_count: &'b mut u64,
    /// Latch: false until the post-join `/biome` command tree has been sent. The
    /// command tree is emitted once, right after the first chunk batch reaches the
    /// client, so it lands after the join-ready prefix (matching the vanilla join
    /// capture in `harness/mineflayer/raw_26_1_2_join_probe.mjs`).
    join_commands_sent: &'b mut bool,
}

fn tick_player_and_chunk_sender(
    stream: &mut TcpStream,
    compression: CompressionState,
    play_state: &mut PlaySessionState,
    context: PlayerTickContext<'_, '_>,
) -> io::Result<()> {
    let PlayerTickContext {
        properties,
        world_root,
        world_seed,
        chunk_cache,
        chunk_pipeline,
        current_chunk_x,
        current_chunk_z,
        chunk_sender,
        chunk_pipeline_stats,
        live_fluid_ticks,
        live_block_ticks,
        world_items,
        world_mobs,
        loaded_chunks,
        world_layout,
        last_player_tick,
        play_tick_count,
        join_commands_sent,
    } = context;
    if last_player_tick.elapsed() < SERVER_TICK_DURATION {
        return Ok(());
    }

    *last_player_tick = Instant::now();
    *play_tick_count = (*play_tick_count).wrapping_add(1);
    let tick_count = *play_tick_count;
    tick_live_world_systems(
        stream,
        compression,
        play_state,
        LiveWorldTickContext {
            tick_count,
            live_fluid_ticks,
            live_block_ticks,
            world_layout,
            world_seed,
            chunk_cache,
            world_items,
            world_mobs,
            loaded_chunks,
            world_root,
        },
    )?;
    let fluid_state =
        detect_play_session_fluid_state(play_state, world_root, world_seed, chunk_cache);
    let water_update = tick_play_session_water(play_state, fluid_state);
    if water_update.air_changed {
        write_play_state_air_supply_packet(stream, compression, play_state)?;
    }
    if water_update.motion_changed {
        write_play_state_motion_packet(stream, compression, play_state)?;
    }
    if tick_play_session_food(
        play_state,
        food_difficulty_from_properties(properties),
        true,
        tick_count,
    ) || water_update.health_changed
    {
        write_play_state_health_packet(stream, compression, play_state)?;
    }

    // Per-tick chunk send drain (Java mirror:
    // MinecraftServer.tickChildren -> chunkSender.sendNextChunks).
    // Sits at the end of the player tick so fluid/entity ticking sees
    // the same chunk snapshot as the chunks being flushed.
    let drained = drain_chunk_sender(
        stream,
        compression,
        chunk_sender,
        chunk_pipeline,
        ChunkPos {
            x: current_chunk_x,
            z: current_chunk_z,
        },
        Some((live_fluid_ticks, tick_count as i64)),
    )?;
    chunk_pipeline_stats.sent_total = chunk_pipeline_stats
        .sent_total
        .saturating_add(drained as u64);
    // Send the `/biome` debug command tree exactly once, right after the first
    // chunk batch reaches the client. This places the commands packet after the
    // join-ready prefix (login..chunk_batch_start) instead of mid-join, matching
    // the vanilla join capture; see `write_join_commands_packet` and
    // `harness/mineflayer/raw_26_1_2_join_probe.mjs`.
    if drained > 0 && !*join_commands_sent {
        write_join_commands_packet(stream, compression)?;
        *join_commands_sent = true;
    }
    maybe_log_chunk_pipeline_stats(
        chunk_pipeline_stats,
        chunk_sender,
        chunk_pipeline,
        tick_count,
    );
    Ok(())
}

struct LiveWorldTickContext<'a, 'b> {
    tick_count: u64,
    live_fluid_ticks: &'b mut LiveFluidTicks,
    live_block_ticks: &'b mut LiveBlockTicks,
    world_layout: &'b WorldLayout,
    world_seed: i64,
    chunk_cache: &'a GeneratedChunkCache,
    world_items: &'a Arc<Mutex<WorldItemEntities>>,
    world_mobs: &'a Arc<Mutex<LiveMobStore>>,
    loaded_chunks: &'a BTreeSet<(i32, i32)>,
    world_root: &'a Path,
}

fn tick_live_world_systems(
    stream: &mut TcpStream,
    compression: CompressionState,
    play_state: &mut PlaySessionState,
    context: LiveWorldTickContext<'_, '_>,
) -> io::Result<()> {
    let tick_count = context.tick_count;
    process_live_fluid_ticks(
        stream,
        compression,
        context.live_fluid_ticks,
        tick_count as i64,
        context.world_layout,
        context.world_seed,
        context.chunk_cache,
    )?;
    super::block_placement_live::process_live_block_ticks(
        stream,
        compression,
        context.live_block_ticks,
        tick_count as i64,
        context.world_layout,
        context.world_seed,
        context.chunk_cache,
        context.world_items,
    )?;
    tick_live_falling_blocks(
        stream,
        compression,
        context.live_fluid_ticks,
        context.live_block_ticks,
        tick_count as i64,
        context.world_layout,
        context.world_seed,
        context.chunk_cache,
        context.world_items,
    )?;
    super::live_mobs::tick_live_mobs_for_client(
        stream,
        compression,
        super::live_mobs::LiveMobClientTickContext {
            store: context.world_mobs,
            loaded_chunks: context.loaded_chunks,
            chunk_cache: context.chunk_cache,
            world_root: context.world_root,
            world_seed: context.world_seed,
            play_state,
            tick_count,
        },
    )?;
    tick_live_block_destroy_progress(
        stream,
        compression,
        play_state,
        LiveDestroyTickContext {
            tick_count,
            live_fluid_ticks: context.live_fluid_ticks,
            live_block_ticks: context.live_block_ticks,
            world_layout: context.world_layout,
            world_seed: context.world_seed,
            chunk_cache: context.chunk_cache,
            world_items: context.world_items,
        },
    )
}

struct LiveDestroyTickContext<'a, 'b> {
    tick_count: u64,
    live_fluid_ticks: &'b mut LiveFluidTicks,
    live_block_ticks: &'b mut LiveBlockTicks,
    world_layout: &'b WorldLayout,
    world_seed: i64,
    chunk_cache: &'a GeneratedChunkCache,
    world_items: &'a Arc<Mutex<WorldItemEntities>>,
}

struct LiveDestroyContext<'a, 'b> {
    game_time: i64,
    live_fluid_ticks: &'b mut LiveFluidTicks,
    live_block_ticks: &'b mut LiveBlockTicks,
    world_layout: &'b WorldLayout,
    world_seed: i64,
    chunk_cache: &'a GeneratedChunkCache,
    world_items: &'a Arc<Mutex<WorldItemEntities>>,
}

fn tick_live_block_destroy_progress(
    stream: &mut TcpStream,
    compression: CompressionState,
    play_state: &mut PlaySessionState,
    context: LiveDestroyTickContext<'_, '_>,
) -> io::Result<()> {
    play_state.block_break_state.game_ticks = context.tick_count as i32;

    if play_state.block_break_state.has_delayed_destroy {
        let pos = play_state.block_break_state.delayed_destroy_pos;
        let block_state = read_live_block_model_at(
            context.chunk_cache,
            context.world_layout,
            context.world_seed,
            pos,
        );
        let state_name = block_state.state_name();
        let physics = crate::block_properties::state_physics_by_name(&state_name);
        if physics.is_none_or(|state| state.is_air) {
            play_state.block_break_state.has_delayed_destroy = false;
            play_state.block_break_state.last_sent_state = -1;
            return write_block_destruction(stream, compression, pos, -1);
        }

        let (tool_speed, has_correct_tool) = live_block_destroy_tool_inputs(play_state, &state_name);
        let Some(progress) = block_destroy_progress_state(
            &play_state.block_break_state,
            physics,
            tool_speed,
            has_correct_tool,
            play_state.block_break_state.delayed_tick_start,
        ) else {
            return Ok(());
        };
        if progress >= 10 {
            play_state.block_break_state.has_delayed_destroy = false;
            return destroy_live_block_at(
                stream,
                compression,
                play_state,
                LiveDestroyContext {
                    game_time: context.tick_count as i64,
                    live_fluid_ticks: context.live_fluid_ticks,
                    live_block_ticks: context.live_block_ticks,
                    world_layout: context.world_layout,
                    world_seed: context.world_seed,
                    chunk_cache: context.chunk_cache,
                    world_items: context.world_items,
                },
                pos,
                true,
            );
        }
        return write_block_destroy_progress_if_changed(
            stream,
            compression,
            &mut play_state.block_break_state,
            pos,
            progress,
        );
    }

    if !play_state.block_break_state.is_destroying {
        return Ok(());
    }

    let pos = play_state.block_break_state.destroy_pos;
    let block_state = read_live_block_model_at(
        context.chunk_cache,
        context.world_layout,
        context.world_seed,
        pos,
    );
    let state_name = block_state.state_name();
    let physics = crate::block_properties::state_physics_by_name(&state_name);
    if physics.is_none_or(|state| state.is_air) {
        play_state.block_break_state.is_destroying = false;
        play_state.block_break_state.last_sent_state = -1;
        return write_block_destruction(stream, compression, pos, -1);
    }

    let (tool_speed, has_correct_tool) = live_block_destroy_tool_inputs(play_state, &state_name);
    let Some(progress) = block_destroy_progress_state(
        &play_state.block_break_state,
        physics,
        tool_speed,
        has_correct_tool,
        play_state.block_break_state.destroy_progress_start,
    ) else {
        return Ok(());
    };
    write_block_destroy_progress_if_changed(
        stream,
        compression,
        &mut play_state.block_break_state,
        pos,
        progress,
    )
}

fn live_block_destroy_tool_inputs(play_state: &PlaySessionState, state_name: &str) -> (f32, bool) {
    let physics = crate::block_properties::state_physics_by_name(state_name);
    let held_item = selected_main_hand_item(play_state);
    let held_item_id = held_item
        .filter(|stack| !stack.is_empty())
        .map(crate::item_stack::ItemStack::item_id);
    let tool_speed = held_item_id.map_or(1.0, |item_id| {
        crate::player_game_mode::item_destroy_speed_for_block(item_id, state_name)
    });
    let has_correct_tool = physics.is_some_and(|state| {
        held_item_id.is_some_and(|item_id| {
            crate::player_game_mode::item_has_correct_tool_for_drops(
                item_id,
                state_name,
                state.requires_correct_tool_for_drops,
            )
        }) || !state.requires_correct_tool_for_drops
    });
    (tool_speed, has_correct_tool)
}

fn block_destroy_progress_state(
    state: &crate::player_game_mode::BlockBreakState,
    physics: Option<&crate::block_properties::StatePhysics>,
    tool_speed: f32,
    has_correct_tool: bool,
    destroy_start_tick: i32,
) -> Option<i32> {
    let physics = physics?;
    let ticks_spent = state.game_ticks - destroy_start_tick;
    let progress = crate::player_game_mode::compute_destroy_progress(
        physics.destroy_speed,
        tool_speed,
        has_correct_tool,
    ) * (ticks_spent + 1) as f32;
    Some((progress * 10.0) as i32)
}

fn write_block_destroy_progress_if_changed<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &mut crate::player_game_mode::BlockBreakState,
    pos: crate::block_update::BlockPos,
    progress: i32,
) -> io::Result<()> {
    if progress == state.last_sent_state {
        return Ok(());
    }
    state.last_sent_state = progress;
    write_block_destruction(writer, compression, pos, progress)
}

struct JoinedPlayLoopTickContext<'a, 'b> {
    properties: &'a ServerProperties,
    world_root: &'a Path,
    world_seed: i64,
    clock: &'a Arc<Mutex<ServerClockManager>>,
    weather: &'a Arc<Mutex<WeatherCycle>>,
    world_items: &'a Arc<Mutex<WorldItemEntities>>,
    world_mobs: &'a Arc<Mutex<LiveMobStore>>,
    chunk_cache: &'a GeneratedChunkCache,
    chunk_pipeline: &'a ChunkPipeline,
    current_chunk_x: i32,
    current_chunk_z: i32,
    loaded_chunks: &'a BTreeSet<(i32, i32)>,
    chunk_sender: &'b mut PlayerChunkSender,
    chunk_pipeline_stats: &'b mut ChunkPipelineSessionStats,
    world_layout: &'b WorldLayout,
    keep_alive: &'b mut KeepAliveState,
    keep_alive_epoch: Instant,
    last_time_sync: &'b mut Instant,
    last_item_tick: &'b mut Instant,
    last_player_tick: &'b mut Instant,
    play_tick_count: &'b mut u64,
    live_fluid_ticks: &'b mut LiveFluidTicks,
    live_block_ticks: &'b mut LiveBlockTicks,
    last_sent_rain_level: &'b mut f32,
    last_sent_thunder_level: &'b mut f32,
    join_commands_sent: &'b mut bool,
}

/// Returns `false` when the session should end (e.g. a keepalive timeout, whose
/// `disconnect.timeout` packet has already been written).
fn tick_joined_play_session_loop(
    stream: &mut TcpStream,
    compression: CompressionState,
    play_state: &mut PlaySessionState,
    context: JoinedPlayLoopTickContext<'_, '_>,
) -> io::Result<bool> {
    let JoinedPlayLoopTickContext {
        properties,
        world_root,
        world_seed,
        clock,
        weather,
        world_items,
        world_mobs,
        chunk_cache,
        chunk_pipeline,
        current_chunk_x,
        current_chunk_z,
        loaded_chunks,
        chunk_sender,
        chunk_pipeline_stats,
        world_layout,
        keep_alive,
        keep_alive_epoch,
        last_time_sync,
        last_item_tick,
        last_player_tick,
        play_tick_count,
        live_fluid_ticks,
        live_block_ticks,
        last_sent_rain_level,
        last_sent_thunder_level,
        join_commands_sent,
    } = context;
    if !tick_keep_alive_and_time(
        stream,
        compression,
        clock,
        keep_alive,
        keep_alive_epoch,
        last_time_sync,
    )? {
        return Ok(false);
    }
    tick_item_entities_for_client(stream, compression, world_items, last_item_tick)?;
    tick_player_and_chunk_sender(
        stream,
        compression,
        play_state,
        PlayerTickContext {
            properties,
            world_root,
            world_seed,
            chunk_cache,
            chunk_pipeline,
            current_chunk_x,
            current_chunk_z,
            chunk_sender,
            chunk_pipeline_stats,
            live_fluid_ticks,
            live_block_ticks,
            world_items,
            world_mobs,
            loaded_chunks,
            world_layout,
            last_player_tick,
            play_tick_count,
            join_commands_sent,
        },
    )?;
    broadcast_weather_if_changed(
        stream,
        compression,
        weather,
        last_sent_rain_level,
        last_sent_thunder_level,
    )?;
    Ok(true)
}

fn persist_play_disconnect_state(
    properties: &ServerProperties,
    world_root: &Path,
    profile_uuid: &str,
    play_state: &mut PlaySessionState,
    world_items: &Arc<Mutex<WorldItemEntities>>,
    chunk_cache: &GeneratedChunkCache,
) {
    // Java: InventoryMenu.removed() clears the crafting grid and returns
    // items to inventory before the player state is persisted.
    play_state.inventory_menu.clear_crafting_to_inventory();
    let _ = save_play_session_state(world_root, profile_uuid, play_state);
    // TODO(live-stats-advancements-persistence): Java ServerPlayer.disconnect ->
    // PlayerList.save() also flushes the player's stats (stats/<uuid>.json via
    // ServerStatsCounter.save) and advancements (advancements/<uuid>.json via
    // PlayerAdvancements.save). The file format is implemented + tested
    // (statistics.rs to_vanilla_json/from_vanilla_json, WorldLayout::save_stats/
    // save_advancements) but is NOT wired here, and stats are never incremented
    // during live play (no .increment() calls in the play loop). Wiring requires
    // live gameplay stat tracking (movement/mining/etc.) + advancement criteria
    // triggers. Blocks CHECKLIST_STORAGE #80 (advancements) and #81 (stats).
    save_world_item_entities(world_root, &lock_status_mutex(world_items));
    // Flush any in-memory block changes (player edits, fluid spreads) that
    // have not reached the periodic flush window; disconnect must not lose work.
    chunk_cache.flush_dirty(
        world_root,
        properties.sync_chunk_writes,
        RegionCompression::from_property_value(&properties.region_file_compression),
    );
}

struct RespawnSessionContext<'a, 'b> {
    properties: &'a ServerProperties,
    world_root: &'a Path,
    world_seed: i64,
    profile_uuid: &'a str,
    chunk_pipeline: &'a ChunkPipeline,
    current_chunk_x: &'b mut i32,
    current_chunk_z: &'b mut i32,
    chunk_batch_radius: i32,
    loaded_chunks: &'b mut BTreeSet<(i32, i32)>,
    chunk_sender: &'b mut PlayerChunkSender,
}

fn handle_respawn_session_update(
    stream: &mut TcpStream,
    compression: CompressionState,
    play_state: &mut PlaySessionState,
    context: RespawnSessionContext<'_, '_>,
) -> io::Result<()> {
    let RespawnSessionContext {
        properties,
        world_root,
        world_seed,
        profile_uuid,
        chunk_pipeline,
        current_chunk_x,
        current_chunk_z,
        chunk_batch_radius,
        loaded_chunks,
        chunk_sender,
    } = context;
    handle_play_respawn_request(
        stream,
        compression,
        play_state,
        properties,
        world_root,
        world_seed,
    )?;
    *current_chunk_x = chunk_coordinate(play_state.x);
    *current_chunk_z = chunk_coordinate(play_state.z);
    *loaded_chunks = chunk_window(*current_chunk_x, *current_chunk_z, chunk_batch_radius);
    // Re-seed the per-session sender for the new spawn location. Pending
    // chunks from before the respawn no longer make sense because the center
    // and visible window changed.
    *chunk_sender = PlayerChunkSender::new(false);
    seed_chunk_window(
        chunk_sender,
        chunk_pipeline,
        *current_chunk_x,
        *current_chunk_z,
        chunk_batch_radius,
    );
    let _ = save_play_session_state(world_root, profile_uuid, play_state);
    Ok(())
}

struct PositionSessionContext<'a, 'b> {
    world_root: &'a Path,
    world_seed: i64,
    profile_uuid: &'a str,
    world_items: &'a Arc<Mutex<WorldItemEntities>>,
    recipe_manager: &'a RecipeManagerModel,
    chunk_pipeline: &'a ChunkPipeline,
    current_chunk_x: &'b mut i32,
    current_chunk_z: &'b mut i32,
    chunk_batch_radius: i32,
    loaded_chunks: &'b mut BTreeSet<(i32, i32)>,
    chunk_sender: &'b mut PlayerChunkSender,
}

fn handle_position_session_update(
    stream: &mut TcpStream,
    compression: CompressionState,
    play_state: &mut PlaySessionState,
    context: PositionSessionContext<'_, '_>,
) -> io::Result<()> {
    let PositionSessionContext {
        world_root,
        world_seed,
        profile_uuid,
        world_items,
        chunk_pipeline,
        current_chunk_x,
        current_chunk_z,
        chunk_batch_radius,
        loaded_chunks,
        chunk_sender,
        recipe_manager,
    } = context;
    let next_chunk_x = chunk_coordinate(play_state.x);
    let next_chunk_z = chunk_coordinate(play_state.z);
    if next_chunk_x != *current_chunk_x || next_chunk_z != *current_chunk_z {
        *current_chunk_x = next_chunk_x;
        *current_chunk_z = next_chunk_z;
        // Diff old/new visible windows: forget chunks leaving the window and
        // enqueue chunks entering it. Java mirror: ChunkMap.applyChunkTrackingView.
        apply_chunk_movement(
            stream,
            compression,
            ChunkMovementContext {
                chunk_sender,
                chunk_pipeline,
                loaded_chunks,
                new_center: ChunkPos {
                    x: *current_chunk_x,
                    z: *current_chunk_z,
                },
                radius: chunk_batch_radius,
                world_root,
                world_seed,
            },
        )?;
    }
    // Hook B: Pickup check — mirrors Player.aiStep() proximity sweep.
    // Spectators cannot pick up items.
    if play_state.game_mode != GameMode::Spectator {
        process_item_pickups(
            stream,
            compression,
            play_state,
            profile_uuid,
            world_items,
            recipe_manager,
        )?;
    }
    Ok(())
}

struct PlayerActionFields {
    action: i32,
    packed_pos: i64,
    sequence: i32,
    x: i32,
    y: i32,
    z: i32,
}

fn read_player_action_fields<R: Read>(reader: &mut R) -> io::Result<PlayerActionFields> {
    let action = read_var_i32(reader)?;
    let mut pos_bytes = [0u8; 8];
    reader.read_exact(&mut pos_bytes)?;
    let packed_pos = i64::from_be_bytes(pos_bytes);
    let mut direction_byte = [0u8; 1];
    reader.read_exact(&mut direction_byte)?;
    let sequence = read_var_i32(reader)?;
    let (x, y, z) = unpack_block_position(packed_pos);
    Ok(PlayerActionFields {
        action,
        packed_pos,
        sequence,
        x,
        y,
        z,
    })
}

struct PlayerActionContext<'a, 'b> {
    world_root: &'a Path,
    world_seed: i64,
    world_layout: &'b WorldLayout,
    chunk_cache: &'a GeneratedChunkCache,
    live_fluid_ticks: &'b mut LiveFluidTicks,
    live_block_ticks: &'b mut LiveBlockTicks,
    play_tick_count: u64,
    world_items: &'a Arc<Mutex<WorldItemEntities>>,
    // Spawn-protection inputs (Java ServerLevel.mayInteract ->
    // DedicatedServer.isUnderSpawnProtection): the player access registry (op
    // status), the player's UUID, and the configured `spawn-protection` radius.
    player_access: &'a Arc<Mutex<PlayerAccess>>,
    profile_uuid: &'a str,
    spawn_protection_radius: u32,
}

fn handle_player_action_packet<R: Read>(
    stream: &mut TcpStream,
    compression: CompressionState,
    input: &mut R,
    play_state: &mut PlaySessionState,
    mut context: PlayerActionContext<'_, '_>,
) -> io::Result<()> {
    let fields = read_player_action_fields(input)?;
    log_player_action_debug(&fields, play_state.game_mode, &context);
    if is_block_break_action(fields.action) {
        let block_pos = crate::block_update::BlockPos {
            x: fields.x,
            y: fields.y,
            z: fields.z,
        };
        // Java ServerPlayerGameMode.handleBlockBreakAction (lines 153-170) checks,
        // in order: reach, then spawn protection, then the break itself.
        if !super::player_creative_packets::is_within_block_interaction_range(play_state, block_pos)
        {
            // Out of reach: Java logs "too far" and does NOT break or correct the
            // client (the client never predicts an out-of-range break). Ignore it.
            write_block_change_ack(stream, compression, fields.sequence)?;
        } else if block_break_above_build_height(fields.y) {
            // Above the world ceiling (Java handleBlockBreakAction "too high"):
            // never break — there is no block to remove and set_block must not run
            // outside the world's vertical bounds.
            write_block_change_ack(stream, compression, fields.sequence)?;
        } else if block_break_is_spawn_protected(&fields, &context) {
            // Non-op breaking inside the spawn-protection radius is denied: the
            // server does NOT change the block and re-sends the real state so the
            // client reverts its predicted break.
            write_block_break_denied(stream, compression, &fields, &context)?;
        } else {
            handle_player_block_action(stream, compression, play_state, &fields, &mut context)?;
        }
    }
    // Java: ServerboundPlayerActionPacket.Action.DROP_ALL_ITEMS = 3,
    //        ServerboundPlayerActionPacket.Action.DROP_ITEM = 4.
    if fields.action == 3 || fields.action == 4 {
        handle_drop_item(
            stream,
            compression,
            play_state,
            context.world_items,
            fields.action == 3,
        )?;
    }
    Ok(())
}

/// The only dimension the live server currently runs, and the dimension
/// `spawn-protection` applies to (Java checks `level.dimension() ==
/// respawnData.dimension()`).
const SPAWN_PROTECTION_DIMENSION: &str = "minecraft:overworld";

/// Pure spawn-protection decision, 1:1 with Java
/// `DedicatedServer.isUnderSpawnProtection` (via `PlayerAccess`). Returns true
/// when breaking `pos` must be denied.
pub(crate) fn spawn_protection_break_denied(
    access: &PlayerAccess,
    radius: u32,
    world_spawn: crate::block_update::BlockPos,
    player_uuid: &str,
    pos: crate::block_update::BlockPos,
) -> bool {
    let protection = crate::player_access::SpawnProtection {
        radius,
        spawn_dimension: SPAWN_PROTECTION_DIMENSION.to_string(),
        spawn_pos: world_spawn,
    };
    access.is_under_spawn_protection(&protection, SPAWN_PROTECTION_DIMENSION, pos, player_uuid)
}

/// Live wrapper around [`spawn_protection_break_denied`]. Cheap in-memory op
/// guards run first; the world spawn (level.dat) is only read when a non-op
/// breaks a block on a server that actually has operators.
fn block_break_is_spawn_protected(
    fields: &PlayerActionFields,
    context: &PlayerActionContext<'_, '_>,
) -> bool {
    if context.spawn_protection_radius == 0 {
        return false;
    }
    {
        let access = lock_status_mutex(context.player_access);
        if !access.has_ops() || access.is_op(context.profile_uuid) {
            return false;
        }
    }
    let spawn = world_spawn_suggestion(context.world_root, context.world_seed);
    let world_spawn = crate::block_update::BlockPos {
        x: spawn.0,
        y: spawn.1,
        z: spawn.2,
    };
    let pos = crate::block_update::BlockPos {
        x: fields.x,
        y: fields.y,
        z: fields.z,
    };
    spawn_protection_break_denied(
        &lock_status_mutex(context.player_access),
        context.spawn_protection_radius,
        world_spawn,
        context.profile_uuid,
        pos,
    )
}

/// Build the network-NBT component for the spawn-protection overlay message,
/// 1:1 with Java `ServerPlayer.sendSpawnProtectionMessage`:
/// `Component.translatable("build.spawn_protection", pos.toShortString())
/// .withStyle(ChatFormatting.RED)`. `Vec3i.toShortString()` is `"x, y, z"`.
fn spawn_protection_message_tag(x: i32, y: i32, z: i32) -> crate::storage::nbt::Tag {
    use crate::storage::nbt::Tag;
    Tag::Compound(vec![
        (
            "translate".to_string(),
            Tag::String("build.spawn_protection".to_string()),
        ),
        (
            "with".to_string(),
            Tag::List(vec![Tag::String(format!("{x}, {y}, {z}"))]),
        ),
        ("color".to_string(), Tag::String("red".to_string())),
    ])
}

/// Send the spawn-protection feedback, 1:1 with Java
/// `ServerPlayer.sendSpawnProtectionMessage` → `sendOverlayMessage` →
/// `sendSystemMessage(message, true)`: a system-chat packet with `overlay = true`
/// (the action-bar slot) carrying the RED `build.spawn_protection` component.
pub(crate) fn write_spawn_protection_message(
    stream: &mut TcpStream,
    compression: CompressionState,
    x: i32,
    y: i32,
    z: i32,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SYSTEM_CHAT_PACKET_ID,
        |payload| {
            ClientboundSystemChatPacket {
                content: spawn_protection_message_tag(x, y, z),
                overlay: true,
            }
            .write(payload)
        },
    )
}

/// Build the network-NBT component for the build-limit overlay message, 1:1 with
/// Java `ServerPlayer.sendBuildLimitMessage`:
/// `Component.translatable(isTooHigh ? "build.tooHigh" : "build.tooLow", limit)
/// .withStyle(ChatFormatting.RED)`.
pub(crate) fn build_limit_message_tag(is_too_high: bool, limit: i32) -> crate::storage::nbt::Tag {
    use crate::storage::nbt::Tag;
    Tag::Compound(vec![
        (
            "translate".to_string(),
            Tag::String(
                if is_too_high {
                    "build.tooHigh"
                } else {
                    "build.tooLow"
                }
                .to_string(),
            ),
        ),
        ("with".to_string(), Tag::List(vec![Tag::Int(limit)])),
        ("color".to_string(), Tag::String("red".to_string())),
    ])
}

/// Send the RED build-limit overlay message (`ServerPlayer.sendBuildLimitMessage`)
/// as a system-chat packet with `overlay = true`.
pub(crate) fn write_build_limit_message(
    stream: &mut TcpStream,
    compression: CompressionState,
    is_too_high: bool,
    limit: i32,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SYSTEM_CHAT_PACKET_ID,
        |payload| {
            ClientboundSystemChatPacket {
                content: build_limit_message_tag(is_too_high, limit),
                overlay: true,
            }
            .write(payload)
        },
    )
}

/// On spawn-protection denial, mirror Java
/// `ServerPlayerGameMode.handleBlockBreakAction`'s spawn-protection branch:
/// 1. ack the action sequence (so the client reverts its predicted break — Java
///    relies on the per-tick BlockChangedAck for this);
/// 2. re-send the real block state for robustness (the block is NOT modified
///    server-side and no drops spawn);
/// 3. send the RED `build.spawn_protection` overlay message
///    (`ServerPlayer.sendSpawnProtectionMessage`).
fn write_block_break_denied(
    stream: &mut TcpStream,
    compression: CompressionState,
    fields: &PlayerActionFields,
    context: &PlayerActionContext<'_, '_>,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID,
        |payload| write_var_i32(payload, fields.sequence),
    )?;
    let chunk = context.chunk_cache.get_or_load(
        fields.x.div_euclid(16),
        fields.z.div_euclid(16),
        context.world_root,
        context.world_seed,
    );
    let block_name = chunk
        .get_block_state(fields.x, fields.y, fields.z)
        .unwrap_or_else(|| "minecraft:air".to_string());
    let state_id = block_state_name_network_id(&block_name).unwrap_or(AIR_BLOCK_STATE_ID);
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_BLOCK_UPDATE_PACKET_ID,
        |payload| {
            payload.write_all(&fields.packed_pos.to_be_bytes())?;
            write_var_i32(payload, state_id)
        },
    )?;
    write_spawn_protection_message(stream, compression, fields.x, fields.y, fields.z)
}

fn log_player_action_debug(
    fields: &PlayerActionFields,
    game_mode: GameMode,
    context: &PlayerActionContext<'_, '_>,
) {
    crate::log::log_debug(&format!(
        "player_action action={} pos=({},{},{}) mode={:?}",
        fields.action, fields.x, fields.y, fields.z, game_mode
    ));
    if fields.action != 0 {
        return;
    }
    // Java ServerPlayerGameMode: START_DESTROY_BLOCK with getDestroyProgress
    // >= 1.0 (destroy_time == 0) -> "insta mine".
    let chunk_pos = ChunkPos {
        x: fields.x.div_euclid(16),
        z: fields.z.div_euclid(16),
    };
    let actual_block = read_block_at(
        context.world_layout,
        context.world_seed,
        chunk_pos,
        fields.x,
        fields.y,
        fields.z,
    );
    let destroy_time = actual_block
        .as_deref()
        .and_then(crate::block_properties::state_physics_by_name)
        .map(|physics| physics.destroy_speed);
    crate::log::log_debug(&format!(
        "instabreak check: actual_block={actual_block:?} destroy_time={destroy_time:?}"
    ));
}

/// Java `Player.blockActionRestricted` (Player.java:188-194): `SPECTATOR` is
/// unconditionally restricted from block actions, so spectators can never break a
/// block. (Adventure mode's break eligibility additionally depends on the held
/// item's `CanDestroy` component, which is not yet modelled here.)
fn spectator_cannot_break(game_mode: GameMode) -> bool {
    game_mode == GameMode::Spectator
}

/// Java `ServerPlayerGameMode.handleBlockBreakAction` (line 154) rejects a break
/// when `pos.getY() > level.getMaxY()`. The single live dimension is the
/// overworld, whose `getMaxY()` = `getMinY() + getHeight() - 1` =
/// `OVERWORLD_MIN_Y + OVERWORLD_LEVEL_HEIGHT - 1`.
fn block_break_above_build_height(y: i32) -> bool {
    y > crate::world::OVERWORLD_MIN_Y + crate::world::OVERWORLD_LEVEL_HEIGHT - 1
}

fn is_block_break_action(action: i32) -> bool {
    matches!(action, 0..=2)
}

fn handle_player_block_action(
    stream: &mut TcpStream,
    compression: CompressionState,
    play_state: &mut PlaySessionState,
    fields: &PlayerActionFields,
    context: &mut PlayerActionContext<'_, '_>,
) -> io::Result<()> {
    if spectator_cannot_break(play_state.game_mode) {
        return write_block_change_ack(stream, compression, fields.sequence);
    }
    let Some(action) = player_block_break_action(fields.action) else {
        return Ok(());
    };
    let block_pos = crate::block_update::BlockPos {
        x: fields.x,
        y: fields.y,
        z: fields.z,
    };
    let block_state = read_live_block_model_at(
        context.chunk_cache,
        context.world_layout,
        context.world_seed,
        block_pos,
    );
    let state_name = block_state.state_name();
    let physics = crate::block_properties::state_physics_by_name(&state_name);
    let block_is_air = physics.is_none_or(|state| state.is_air);
    let block_hardness = physics.map_or(0.0, |state| state.destroy_speed);
    let held_item = selected_main_hand_item(play_state);
    let held_item_id = held_item
        .filter(|stack| !stack.is_empty())
        .map(crate::item_stack::ItemStack::item_id);
    let tool_speed = held_item_id.map_or(1.0, |item_id| {
        crate::player_game_mode::item_destroy_speed_for_block(item_id, &state_name)
    });
    let has_correct_tool = physics.is_some_and(|state| {
        held_item_id.is_some_and(|item_id| {
            crate::player_game_mode::item_has_correct_tool_for_drops(
                item_id,
                &state_name,
                state.requires_correct_tool_for_drops,
            )
        }) || !state.requires_correct_tool_for_drops
    });
    play_state.block_break_state.game_ticks = context.play_tick_count as i32;
    let outcome = crate::player_game_mode::handle_block_break_action(
        &mut play_state.block_break_state,
        &crate::player_game_mode::BlockBreakInputContext {
            pos: block_pos,
            action,
            max_y: crate::world::OVERWORLD_MIN_Y + crate::world::OVERWORLD_LEVEL_HEIGHT - 1,
            sequence: fields.sequence,
            game_mode: player_game_mode(play_state.game_mode),
            within_reach: true,
            spawn_protected: false,
            may_interact: true,
            block_action_restricted: false,
            block_hardness,
            tool_speed,
            has_correct_tool,
            block_is_air,
        },
    );
    match outcome {
        crate::player_game_mode::BlockBreakOutcome::InstantBreak
        | crate::player_game_mode::BlockBreakOutcome::Destroy => {
            handle_player_block_break(stream, compression, play_state, fields, context)
        }
        crate::player_game_mode::BlockBreakOutcome::Progress(progress) => {
            write_block_change_ack(stream, compression, fields.sequence)?;
            write_block_destruction(stream, compression, block_pos, progress)
        }
        crate::player_game_mode::BlockBreakOutcome::ProgressReset
        | crate::player_game_mode::BlockBreakOutcome::Aborted => {
            write_block_change_ack(stream, compression, fields.sequence)?;
            write_block_destruction(stream, compression, block_pos, -1)
        }
        crate::player_game_mode::BlockBreakOutcome::Denied(_) => {
            write_block_break_denied(stream, compression, fields, context)
        }
    }
}

fn player_block_break_action(action: i32) -> Option<crate::player_game_mode::BlockBreakAction> {
    match action {
        0 => Some(crate::player_game_mode::BlockBreakAction::Start),
        1 => Some(crate::player_game_mode::BlockBreakAction::Abort),
        2 => Some(crate::player_game_mode::BlockBreakAction::Stop),
        _ => None,
    }
}

fn player_game_mode(game_mode: GameMode) -> crate::player_game_mode::PlayerGameMode {
    match game_mode {
        GameMode::Survival => crate::player_game_mode::PlayerGameMode::Survival,
        GameMode::Creative => crate::player_game_mode::PlayerGameMode::Creative,
        GameMode::Adventure => crate::player_game_mode::PlayerGameMode::Adventure,
        GameMode::Spectator => crate::player_game_mode::PlayerGameMode::Spectator,
    }
}

fn selected_main_hand_item(state: &PlaySessionState) -> Option<&ItemStack> {
    let slot = usize::try_from(state.selected_slot).ok()?;
    if slot >= HOTBAR_SIZE {
        return None;
    }
    Some(state.inventory_menu.player_inventory().get(slot))
}

fn write_block_destruction<W: Write>(
    _writer: &mut W,
    _compression: CompressionState,
    _pos: crate::block_update::BlockPos,
    _progress: i32,
) -> io::Result<()> {
    // Java `ServerLevel.destroyBlockProgress` broadcasts cracking overlays to
    // nearby players except the player whose entity id owns the destroy action.
    // VibeCraft currently has a single live player stream, so echoing the packet
    // here fights the client's local mining animation and causes crack resets.
    Ok(())
}

fn write_block_update_at<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    pos: crate::block_update::BlockPos,
    state_id: i32,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_BLOCK_UPDATE_PACKET_ID,
        |payload| {
            payload.write_all(&block_pos_as_long(pos.x, pos.y, pos.z).to_be_bytes())?;
            write_var_i32(payload, state_id)
        },
    )
}

fn should_drop_block_loot<'a>(
    block_name: &str,
    held_item: Option<&'a ItemStack>,
) -> (bool, Option<&'a str>) {
    let held_item_id = held_item
        .filter(|stack| !stack.is_empty())
        .map(crate::item_stack::ItemStack::item_id);
    let Some(physics) = crate::block_properties::state_physics_by_name(block_name) else {
        return (true, held_item_id);
    };
    let correct = held_item_id.is_some_and(|item_id| {
        crate::player_game_mode::item_has_correct_tool_for_drops(
            item_id,
            block_name,
            physics.requires_correct_tool_for_drops,
        )
    }) || !physics.requires_correct_tool_for_drops;
    (correct, held_item_id)
}

fn handle_player_block_break(
    stream: &mut TcpStream,
    compression: CompressionState,
    play_state: &mut PlaySessionState,
    fields: &PlayerActionFields,
    context: &mut PlayerActionContext<'_, '_>,
) -> io::Result<()> {
    // Reach, spawn protection, spectator-no-break, and Java destroy-progress
    // timing are enforced before this handler runs. Adventure-mode CanDestroy
    // remains deferred until item components are modelled for that path.
    write_block_break_ack_and_air(stream, compression, fields, play_state.game_mode)?;
    let block_pos = crate::block_update::BlockPos {
        x: fields.x,
        y: fields.y,
        z: fields.z,
    };
    destroy_live_block_at(
        stream,
        compression,
        play_state,
        LiveDestroyContext {
            game_time: context.play_tick_count as i64,
            live_fluid_ticks: context.live_fluid_ticks,
            live_block_ticks: context.live_block_ticks,
            world_layout: context.world_layout,
            world_seed: context.world_seed,
            chunk_cache: context.chunk_cache,
            world_items: context.world_items,
        },
        block_pos,
        false,
    )
}

fn destroy_live_block_at(
    stream: &mut TcpStream,
    compression: CompressionState,
    play_state: &mut PlaySessionState,
    context: LiveDestroyContext<'_, '_>,
    block_pos: crate::block_update::BlockPos,
    send_air_update: bool,
) -> io::Result<()> {
    if send_air_update {
        write_block_update_at(stream, compression, block_pos, AIR_BLOCK_STATE_ID)?;
    }
    // Java mirror: ServerLevel.removeBlock -> LevelChunk.setBlockState.
    // Mutates the in-memory chunk and marks it unsaved; persistence happens
    // later via the periodic flush thread.
    let block_name = context.chunk_cache.set_block(
        context.world_layout.root(),
        context.world_seed,
        block_pos,
        "minecraft:air",
    );
    schedule_neighbor_fluids(
        context.live_fluid_ticks,
        context.game_time,
        context.world_layout,
        context.world_seed,
        context.chunk_cache,
        block_pos,
    );
    // Java Block.playerWillDestroy (double-block halves) + the
    // Level.updateNeighborShapes cascade for the cleared position.
    let broken_state = block_name
        .as_deref()
        .map(crate::block_behavior::BlockStateModel::new)
        .map(|state| {
            // Reparse name[props] into a model for the counterpart lookup.
            match block_name.as_deref().and_then(|name| name.split_once('[')) {
                Some((base, raw_properties)) => {
                    let mut parsed = crate::block_behavior::BlockStateModel::new(base);
                    for pair in raw_properties.trim_end_matches(']').split(',') {
                        if let Some((key, value)) = pair.split_once('=') {
                            parsed = parsed.with_property(key.trim(), value.trim());
                        }
                    }
                    parsed
                }
                None => state,
            }
        });
    let mut cascade = super::block_placement_live::LiveCascade {
        layout: context.world_layout,
        seed: context.world_seed,
        cache: context.chunk_cache,
        fluid_ticks: context.live_fluid_ticks,
        block_ticks: context.live_block_ticks,
        game_time: context.game_time,
        random_roll: ((context.game_time as i32) ^ block_pos.x ^ block_pos.z).rem_euclid(40),
    };
    super::block_placement_live::run_block_break_aftermath(
        stream,
        compression,
        &mut cascade,
        block_pos,
        broken_state.as_ref(),
    )?;
    crate::log::log_debug(&format!(
        "block break at ({},{},{}) block={:?} game_mode={:?}",
        block_pos.x, block_pos.y, block_pos.z, block_name, play_state.game_mode
    ));
    if play_state.game_mode != GameMode::Creative {
        if damage_main_hand_tool_after_block_break(play_state, block_name.as_deref()) {
            write_inventory_menu_full_sync(stream, compression, play_state)?;
        }
        spawn_block_break_drops(
            stream,
            compression,
            context.world_items,
            play_state,
            block_pos,
            block_name,
        )?;
    }
    Ok(())
}

fn damage_main_hand_tool_after_block_break(
    play_state: &mut PlaySessionState,
    block_name: Option<&str>,
) -> bool {
    let Some(block_name) = block_name else {
        return false;
    };
    if crate::block_properties::state_physics_by_name(block_name)
        .is_none_or(|physics| physics.destroy_speed == 0.0)
    {
        return false;
    }
    let Ok(slot) = usize::try_from(play_state.selected_slot) else {
        return false;
    };
    if slot >= HOTBAR_SIZE {
        return false;
    }
    let mut stack = play_state.inventory_menu.player_inventory().get(slot).clone();
    if stack.is_empty() || !stack.is_damageable_item() {
        return false;
    }
    let Some(damage) = crate::player_game_mode::item_tool_damage_per_block(stack.item_id()) else {
        return false;
    };
    if damage == 0 {
        return false;
    }

    stack.set_damage_value(stack.damage_value().saturating_add(damage));
    if stack.is_broken() {
        stack = ItemStack::empty();
    }
    play_state
        .inventory_menu
        .player_inventory_mut()
        .set(slot, stack);
    true
}

fn write_block_break_ack_and_air(
    stream: &mut TcpStream,
    compression: CompressionState,
    fields: &PlayerActionFields,
    game_mode: GameMode,
) -> io::Result<()> {
    // Packet ordering rationale:
    //
    // Java defers BlockChangedAck to the start of the next server tick. Our
    // server is synchronous; sending the ack first lets the client commit
    // block prediction before AddEntity arrives, so drops spawn into confirmed AIR.
    if crate::log::global_level() >= crate::log::LogLevel::Trace {
        crate::log::log_trace(&format!(
            "block break seq={} pos=({},{},{}) action={} game_mode={:?}",
            fields.sequence, fields.x, fields.y, fields.z, fields.action, game_mode
        ));
        crate::log::log_trace(&format!(
            "sending BLOCK_CHANGED_ACK seq={}",
            fields.sequence
        ));
    }
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID,
        |payload| write_var_i32(payload, fields.sequence),
    )?;
    if crate::log::global_level() >= crate::log::LogLevel::Trace {
        crate::log::log_trace(&format!(
            "sending BLOCK_UPDATE pos=({},{},{}) new_state=AIR",
            fields.x, fields.y, fields.z
        ));
    }
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_BLOCK_UPDATE_PACKET_ID,
        |payload| {
            payload.write_all(&fields.packed_pos.to_be_bytes())?;
            write_var_i32(payload, AIR_BLOCK_STATE_ID)
        },
    )
}

/// Per-tick falling-block simulation (Java `FallingBlockEntity.tick`).
#[allow(clippy::too_many_arguments)]
fn tick_live_falling_blocks(
    stream: &mut TcpStream,
    compression: CompressionState,
    live_fluid_ticks: &mut LiveFluidTicks,
    live_block_ticks: &mut LiveBlockTicks,
    game_time: i64,
    world_layout: &WorldLayout,
    world_seed: i64,
    chunk_cache: &GeneratedChunkCache,
    world_items: &Arc<Mutex<WorldItemEntities>>,
) -> io::Result<()> {
    let mut cascade = super::block_placement_live::LiveCascade {
        layout: world_layout,
        seed: world_seed,
        cache: chunk_cache,
        fluid_ticks: live_fluid_ticks,
        block_ticks: live_block_ticks,
        game_time,
        random_roll: (game_time as i32).rem_euclid(40),
    };
    super::block_placement_live::tick_falling_blocks(stream, compression, &mut cascade, world_items)
}

fn spawn_block_break_drops(
    stream: &mut TcpStream,
    compression: CompressionState,
    world_items: &Arc<Mutex<WorldItemEntities>>,
    play_state: &PlaySessionState,
    block_pos: crate::block_update::BlockPos,
    block_name: Option<String>,
) -> io::Result<()> {
    let loot_seed = (block_pos.x as u64).wrapping_mul(0x9E37_79B9)
        ^ (block_pos.y as u64).wrapping_mul(0x6C62_272E)
        ^ (block_pos.z as u64).wrapping_mul(0x517C_C1B7);
    let held_item = selected_main_hand_item(play_state);
    let drops = block_name
        .as_deref()
        .map(|name| {
            let (correct_tool, tool) = should_drop_block_loot(name, held_item);
            if !correct_tool {
                Vec::new()
            } else {
                evaluate_block_loot_with_tool(name, loot_seed, tool, correct_tool)
            }
        })
        .unwrap_or_default();
    for (item_name, count) in drops {
        let Some(item_pid) = item_protocol_id(item_name) else {
            continue;
        };
        let eid = lock_status_mutex(world_items).alloc_entity_id();
        // Java: ItemEntity constructor sets initial velocity
        // (random*0.2-0.1, 0.2, random*0.2-0.1).
        let item = DroppedItem {
            entity_id: eid,
            item: item_name,
            count,
            x: block_pos.x as f64 + 0.5,
            y: block_pos.y as f64 + 0.5,
            z: block_pos.z as f64 + 0.5,
            vel_x: pseudo_rand_f32(eid, 0) as f64 * 0.2 - 0.1,
            vel_y: 0.2,
            vel_z: pseudo_rand_f32(eid, 1) as f64 * 0.2 - 0.1,
            pickup_delay: DEFAULT_PICKUP_DELAY,
            age: 0,
            target_uuid: None,
        };
        write_item_entity_spawn_packets(stream, compression, &item, item_pid)?;
        lock_status_mutex(world_items).entities.push(item);
    }
    Ok(())
}

struct InventoryPacketContext<'a, 'b> {
    recipe_manager: &'a RecipeManagerModel,
    world_layout: &'b WorldLayout,
    world_seed: i64,
    chunk_cache: &'a GeneratedChunkCache,
    profile_name: &'a str,
}

fn try_handle_inventory_packet<R: Read>(
    stream: &mut TcpStream,
    compression: CompressionState,
    input: &mut R,
    packet_id: i32,
    play_state: &mut PlaySessionState,
    context: InventoryPacketContext<'_, '_>,
) -> io::Result<bool> {
    match packet_id {
        SERVERBOUND_CONTAINER_CLICK_PACKET_ID => {
            handle_container_click_packet(stream, compression, input, play_state, context)?;
        }
        SERVERBOUND_PICK_ITEM_FROM_BLOCK_PACKET_ID => {
            handle_pick_item_from_block_packet(stream, compression, input, play_state, &context)?;
        }
        SERVERBOUND_PICK_ITEM_FROM_ENTITY_PACKET_ID => {
            handle_pick_item_from_entity_packet(stream, compression, input, play_state)?;
        }
        SERVERBOUND_EDIT_BOOK_PACKET_ID => {
            handle_edit_book_packet(stream, compression, input, play_state, context.profile_name)?;
        }
        SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID => {
            handle_set_creative_mode_slot_packet(stream, compression, input, play_state)?;
        }
        SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS_PACKET_ID => {
            let packet = ServerboundRecipeBookChangeSettingsPacket::read(input)?;
            apply_recipe_book_settings_packet(play_state, packet);
        }
        SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE_PACKET_ID => {
            let packet = ServerboundRecipeBookSeenRecipePacket::read(input)?;
            apply_recipe_book_seen_recipe_packet(
                play_state,
                packet,
                context.recipe_manager.recipe_map(),
            );
        }
        SERVERBOUND_PLACE_RECIPE_PACKET_ID => {
            handle_place_recipe_packet(
                stream,
                compression,
                input,
                play_state,
                context.recipe_manager,
            )?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn handle_container_click_packet<R: Read>(
    stream: &mut TcpStream,
    compression: CompressionState,
    input: &mut R,
    play_state: &mut PlaySessionState,
    context: InventoryPacketContext<'_, '_>,
) -> io::Result<()> {
    // Java: ServerGamePacketListenerImpl.handleContainerClick()
    let Ok(click) = ServerboundContainerClickPacket::read(input) else {
        return Ok(());
    };
    if click.container_id != 0 {
        let Some(mut active_menu) = play_state.active_block_menu.take() else {
            return Ok(());
        };
        let instructions = active_menu.handle_click(
            &click,
            play_state,
            context.world_layout,
            context.world_seed,
            context.chunk_cache,
        );
        play_state.active_block_menu = Some(active_menu);
        for instruction in instructions {
            write_container_click_instruction(
                stream,
                compression,
                instruction,
                context.recipe_manager,
            )?;
        }
        return Ok(());
    }
    let instructions = handle_container_click(
        &click,
        &mut play_state.container_state_id,
        &mut play_state.inventory_menu,
        &mut play_state.carried_item,
    );
    for instruction in instructions {
        write_container_click_instruction(
            stream,
            compression,
            instruction,
            context.recipe_manager,
        )?;
    }
    Ok(())
}

fn write_container_click_instruction(
    stream: &mut TcpStream,
    compression: CompressionState,
    instruction: PlayInstruction,
    recipe_manager: &RecipeManagerModel,
) -> io::Result<()> {
    match instruction {
        PlayInstruction::ContainerSetSlot(packet) => write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID,
            |payload| packet.write(payload),
        ),
        PlayInstruction::SetCursorItem(packet) => write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID,
            |payload| packet.write(payload),
        ),
        PlayInstruction::RecipesUnlocked(ids) => {
            let Some(packet) = build_recipe_book_add(&ids, recipe_manager.recipe_map()) else {
                return Ok(());
            };
            write_framed_packet_with_compression(
                stream,
                compression,
                CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID,
                |payload| packet.write(payload),
            )
        }
        _ => Ok(()),
    }
}

fn handle_pick_item_from_block_packet<R: Read>(
    stream: &mut TcpStream,
    compression: CompressionState,
    input: &mut R,
    play_state: &mut PlaySessionState,
    context: &InventoryPacketContext<'_, '_>,
) -> io::Result<()> {
    let packet = ServerboundPickItemFromBlockPacket::read(input)?;
    let outcome = super::player_creative_packets::apply_pick_item_from_block_packet(
        play_state,
        packet,
        context.world_layout,
        context.chunk_cache,
    );
    write_pick_item_outcome(stream, compression, play_state, outcome)
}

fn handle_pick_item_from_entity_packet<R: Read>(
    stream: &mut TcpStream,
    compression: CompressionState,
    input: &mut R,
    play_state: &mut PlaySessionState,
) -> io::Result<()> {
    let packet = ServerboundPickItemFromEntityPacket::read(input)?;
    let outcome =
        super::player_creative_packets::apply_pick_item_from_entity_packet(play_state, packet);
    write_pick_item_outcome(stream, compression, play_state, outcome)
}

fn write_pick_item_outcome(
    stream: &mut TcpStream,
    compression: CompressionState,
    play_state: &mut PlaySessionState,
    outcome: super::player_creative_packets::PickItemOutcome,
) -> io::Result<()> {
    let super::player_creative_packets::PickItemOutcome::Picked { inventory_changed } = outcome
    else {
        return Ok(());
    };
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_HELD_SLOT_PACKET_ID,
        |payload| {
            ClientboundSetHeldSlotPacket {
                slot: play_state.selected_slot,
            }
            .write(payload)
        },
    )?;
    if inventory_changed {
        play_state.container_state_id = play_state.container_state_id.wrapping_add(1);
        write_inventory_menu_full_sync(stream, compression, play_state)?;
    }
    Ok(())
}

fn handle_edit_book_packet<R: Read>(
    stream: &mut TcpStream,
    compression: CompressionState,
    input: &mut R,
    play_state: &mut PlaySessionState,
    profile_name: &str,
) -> io::Result<()> {
    let packet = ServerboundEditBookPacket::read(input)?;
    if super::player_book_packets::apply_edit_book_packet(play_state, packet, profile_name) {
        play_state.container_state_id = play_state.container_state_id.wrapping_add(1);
        write_inventory_menu_full_sync(stream, compression, play_state)?;
    }
    Ok(())
}

fn handle_set_creative_mode_slot_packet<R: Read>(
    stream: &mut TcpStream,
    compression: CompressionState,
    input: &mut R,
    play_state: &mut PlaySessionState,
) -> io::Result<()> {
    let packet = ServerboundSetCreativeModeSlotPacket::read(input)?;
    let Some(slot_update) =
        super::player_creative_packets::apply_set_creative_mode_slot_packet(play_state, packet)
    else {
        return Ok(());
    };
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID,
        |payload| slot_update.write(payload),
    )
}

fn handle_place_recipe_packet<R: Read>(
    stream: &mut TcpStream,
    compression: CompressionState,
    input: &mut R,
    play_state: &mut PlaySessionState,
    recipe_manager: &RecipeManagerModel,
) -> io::Result<()> {
    let packet = ServerboundPlaceRecipePacket::read(input)?;
    if packet.container_id == 0
        && apply_place_recipe_packet(play_state, packet, recipe_manager.recipe_map())
    {
        write_inventory_menu_full_sync(stream, compression, play_state)?;
    }
    Ok(())
}

struct PlayDisconnectContext<'a, 'b> {
    properties: &'a ServerProperties,
    world_root: &'a Path,
    profile_uuid: &'a str,
    play_state: &'b mut PlaySessionState,
    world_items: &'a Arc<Mutex<WorldItemEntities>>,
    chunk_cache: &'a GeneratedChunkCache,
}

enum PlayPacketReadOutcome {
    Packet(Vec<u8>),
    Continue,
    EndSession,
}

fn read_play_packet_or_handle_disconnect(
    stream: &mut TcpStream,
    compression: CompressionState,
    rate_limiter: &mut PacketRateLimiter,
    context: PlayDisconnectContext<'_, '_>,
) -> io::Result<PlayPacketReadOutcome> {
    match read_packet_with_compression(stream, compression) {
        Ok(packet) => {
            if let PacketRateDecision::Kick { reason } = rate_limiter.record_packet(Instant::now())
            {
                persist_play_disconnect_state(
                    context.properties,
                    context.world_root,
                    context.profile_uuid,
                    context.play_state,
                    context.world_items,
                    context.chunk_cache,
                );
                write_translatable_play_disconnect(stream, compression, &reason)?;
                return Ok(PlayPacketReadOutcome::EndSession);
            }
            Ok(PlayPacketReadOutcome::Packet(packet))
        }
        Err(err)
            if matches!(
                err.kind(),
                io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
            ) =>
        {
            if let PacketRateDecision::Kick { reason } = rate_limiter.tick(Instant::now()) {
                // Java: RateKickingConnection sends a common disconnect after the
                // per-second average crosses the configured threshold.
                persist_play_disconnect_state(
                    context.properties,
                    context.world_root,
                    context.profile_uuid,
                    context.play_state,
                    context.world_items,
                    context.chunk_cache,
                );
                write_translatable_play_disconnect(stream, compression, &reason)?;
                return Ok(PlayPacketReadOutcome::EndSession);
            }
            Ok(PlayPacketReadOutcome::Continue)
        }
        Err(err)
            if matches!(
                err.kind(),
                io::ErrorKind::UnexpectedEof | io::ErrorKind::ConnectionReset
            ) =>
        {
            persist_play_disconnect_state(
                context.properties,
                context.world_root,
                context.profile_uuid,
                context.play_state,
                context.world_items,
                context.chunk_cache,
            );
            Ok(PlayPacketReadOutcome::EndSession)
        }
        Err(err) => Err(err),
    }
}

fn write_translatable_play_disconnect(
    stream: &mut TcpStream,
    compression: CompressionState,
    reason: &str,
) -> io::Result<()> {
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
    )
}

struct DecodedPlayPacketContext<'a, 'b> {
    properties: &'a ServerProperties,
    player_access: &'a Arc<Mutex<PlayerAccess>>,
    world_root: &'a Path,
    world_seed: i64,
    profile: &'a NameAndId,
    recipe_manager: &'a RecipeManagerModel,
    world_layout: &'b WorldLayout,
    chunk_cache: &'a GeneratedChunkCache,
    chunk_pipeline: &'a ChunkPipeline,
    world_items: &'a Arc<Mutex<WorldItemEntities>>,
    world_mobs: &'a Arc<Mutex<LiveMobStore>>,
    weather: &'a Arc<Mutex<WeatherCycle>>,
    current_chunk_x: &'b mut i32,
    current_chunk_z: &'b mut i32,
    chunk_batch_radius: i32,
    loaded_chunks: &'b mut BTreeSet<(i32, i32)>,
    chunk_sender: &'b mut PlayerChunkSender,
    live_fluid_ticks: &'b mut LiveFluidTicks,
    live_block_ticks: &'b mut LiveBlockTicks,
    play_tick_count: u64,
    /// The player's registry guard, used to propagate play-phase
    /// `ClientInformation` listing-preference changes to the status sample.
    active_login: &'a ActiveLoginGuard,
    /// Keepalive tracker, so a serverbound `KeepAlive` response can be validated
    /// against the pending challenge (Java `handleKeepAlive`).
    keep_alive: &'b mut KeepAliveState,
    keep_alive_epoch: Instant,
}

enum PlayPacketDispatchOutcome {
    Continue,
    EndSession,
}

struct JoinedPlayPacketStepContext<'a, 'b> {
    properties: &'a ServerProperties,
    player_access: &'a Arc<Mutex<PlayerAccess>>,
    world_root: &'a Path,
    world_seed: i64,
    profile: &'a NameAndId,
    recipe_manager: &'a RecipeManagerModel,
    world_layout: &'b WorldLayout,
    chunk_cache: &'a GeneratedChunkCache,
    chunk_pipeline: &'a ChunkPipeline,
    world_items: &'a Arc<Mutex<WorldItemEntities>>,
    world_mobs: &'a Arc<Mutex<LiveMobStore>>,
    weather: &'a Arc<Mutex<WeatherCycle>>,
    current_chunk_x: &'b mut i32,
    current_chunk_z: &'b mut i32,
    chunk_batch_radius: i32,
    loaded_chunks: &'b mut BTreeSet<(i32, i32)>,
    chunk_sender: &'b mut PlayerChunkSender,
    live_fluid_ticks: &'b mut LiveFluidTicks,
    live_block_ticks: &'b mut LiveBlockTicks,
    play_tick_count: u64,
    active_login: &'a ActiveLoginGuard,
    keep_alive: &'b mut KeepAliveState,
    keep_alive_epoch: Instant,
}

fn read_and_dispatch_joined_play_packet(
    stream: &mut TcpStream,
    compression: CompressionState,
    rate_limiter: &mut PacketRateLimiter,
    play_state: &mut PlaySessionState,
    context: JoinedPlayPacketStepContext<'_, '_>,
) -> io::Result<PlayPacketDispatchOutcome> {
    let read_outcome = read_play_packet_or_handle_disconnect(
        stream,
        compression,
        rate_limiter,
        PlayDisconnectContext {
            properties: context.properties,
            world_root: context.world_root,
            profile_uuid: &context.profile.uuid,
            play_state,
            world_items: context.world_items,
            chunk_cache: context.chunk_cache,
        },
    )?;
    match read_outcome {
        PlayPacketReadOutcome::Packet(packet) => {
            handle_decoded_play_packet(stream, compression, packet, play_state, context.decoded())
        }
        PlayPacketReadOutcome::Continue => Ok(PlayPacketDispatchOutcome::Continue),
        PlayPacketReadOutcome::EndSession => Ok(PlayPacketDispatchOutcome::EndSession),
    }
}

fn handle_decoded_play_packet(
    stream: &mut TcpStream,
    compression: CompressionState,
    packet: Vec<u8>,
    play_state: &mut PlaySessionState,
    context: DecodedPlayPacketContext<'_, '_>,
) -> io::Result<PlayPacketDispatchOutcome> {
    let mut input = Cursor::new(packet);
    let packet_id = read_var_i32(&mut input)?;
    let session_update = update_play_session_state(packet_id, &mut input, play_state)?;
    if session_update.health_changed {
        write_play_state_health_packet(stream, compression, play_state)?;
    }
    if session_update.respawn_requested {
        handle_respawn_session_update(stream, compression, play_state, context.respawn())?;
        return Ok(PlayPacketDispatchOutcome::Continue);
    }
    if session_update.position_changed {
        handle_position_session_update(stream, compression, play_state, context.position())?;
        return Ok(PlayPacketDispatchOutcome::Continue);
    }
    if packet_id == SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID {
        write_command_suggestions_response(stream, compression, &mut input)?;
    } else if packet_id == SERVERBOUND_CHAT_PACKET_ID {
        handle_chat_packet(stream, compression, &mut input, context.profile)?;
    } else if packet_id == SERVERBOUND_CHAT_COMMAND_PACKET_ID
        || packet_id == SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID
    {
        handle_chat_command_packet(
            stream,
            compression,
            &mut input,
            packet_id == SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID,
            ChatCommandContext {
                profile: context.profile,
                play_state,
                properties: context.properties,
                player_access: context.player_access,
                world_seed: context.world_seed,
                weather: context.weather,
                active_login: context.active_login,
            },
        )?;
    } else if packet_id == SERVERBOUND_USE_ITEM_ON_PACKET_ID {
        let packet = ServerboundUseItemOnPacket::read(&mut input)?;
        handle_use_item_on(
            stream,
            compression,
            play_state,
            context.use_item_on(),
            &packet,
        )?;
    } else if packet_id == SERVERBOUND_PLAYER_ACTION_PACKET_ID {
        handle_player_action_packet(
            stream,
            compression,
            &mut input,
            play_state,
            context.action(),
        )?;
    } else if packet_id == SERVERBOUND_ATTACK_PACKET_ID {
        let packet = ServerboundAttackPacket::read(&mut input)?;
        super::live_mobs::handle_live_mob_attack(
            stream,
            compression,
            context.world_mobs,
            packet,
            play_state,
        )?;
    } else if try_handle_inventory_packet(
        stream,
        compression,
        &mut input,
        packet_id,
        play_state,
        context.inventory(),
    )? {
    } else if packet_id == SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID {
        handle_chunk_batch_received_packet(&mut input, context.chunk_sender)?;
    } else if packet_id == SERVERBOUND_CLIENT_INFORMATION_PACKET_ID {
        // A mid-session settings change re-sends ClientInformation; propagate the
        // listing preference to the status sample (Java
        // `ServerGamePacketListenerImpl.handleClientInformation` →
        // `ServerPlayer.updateOptions`).
        let packet = ServerboundClientInformationPacket::read(&mut input)?;
        context
            .active_login
            .set_allows_listing(packet.information.allows_listing);
    } else if packet_id == SERVERBOUND_CUSTOM_PAYLOAD_PACKET_ID {
        // Parse + validate (the codec enforces the 32767-byte serverbound limit and
        // the `minecraft:brand` channel) then discard. Java
        // `ServerCommonPacketListenerImpl.handleCustomPayload` is an empty no-op; the
        // value is the decode-side validation, so a malformed/oversized payload
        // closes the connection (the read errors) rather than reaching a handler.
        let _ = ServerboundCustomPayloadPacket::read(&mut input)?;
    } else if packet_id == SERVERBOUND_KEEP_ALIVE_PACKET_ID {
        // Validate the keepalive response against the pending challenge (Java
        // `ServerCommonPacketListenerImpl.handleKeepAlive`): a matching id clears the
        // pending flag and updates latency; a stale/unsolicited id disconnects with
        // `disconnect.timeout`.
        let packet = ServerboundKeepAlivePacket::read(&mut input)?;
        let now_ms = context.keep_alive_epoch.elapsed().as_millis() as u64;
        if let KeepAliveTick::Disconnect = context.keep_alive.handle_response(packet, now_ms, false)
        {
            write_disconnect_component(stream, compression, "disconnect.timeout")?;
            return Ok(PlayPacketDispatchOutcome::EndSession);
        }
    } else if !play_packet_is_handled_after_state_update(packet_id) {
        persist_play_disconnect_state(
            context.properties,
            context.world_root,
            &context.profile.uuid,
            play_state,
            context.world_items,
            context.chunk_cache,
        );
        write_unexpected_play_packet_disconnect(stream, compression, packet_id)?;
        return Ok(PlayPacketDispatchOutcome::EndSession);
    }
    Ok(PlayPacketDispatchOutcome::Continue)
}

impl<'a, 'b> JoinedPlayPacketStepContext<'a, 'b> {
    fn decoded(self) -> DecodedPlayPacketContext<'a, 'b> {
        DecodedPlayPacketContext {
            properties: self.properties,
            player_access: self.player_access,
            world_root: self.world_root,
            world_seed: self.world_seed,
            profile: self.profile,
            recipe_manager: self.recipe_manager,
            world_layout: self.world_layout,
            chunk_cache: self.chunk_cache,
            chunk_pipeline: self.chunk_pipeline,
            world_items: self.world_items,
            world_mobs: self.world_mobs,
            weather: self.weather,
            live_block_ticks: self.live_block_ticks,
            current_chunk_x: self.current_chunk_x,
            current_chunk_z: self.current_chunk_z,
            chunk_batch_radius: self.chunk_batch_radius,
            loaded_chunks: self.loaded_chunks,
            chunk_sender: self.chunk_sender,
            live_fluid_ticks: self.live_fluid_ticks,
            play_tick_count: self.play_tick_count,
            active_login: self.active_login,
            keep_alive: self.keep_alive,
            keep_alive_epoch: self.keep_alive_epoch,
        }
    }
}

impl<'a, 'b> DecodedPlayPacketContext<'a, 'b> {
    fn respawn(self) -> RespawnSessionContext<'a, 'b> {
        RespawnSessionContext {
            properties: self.properties,
            world_root: self.world_root,
            world_seed: self.world_seed,
            profile_uuid: &self.profile.uuid,
            chunk_pipeline: self.chunk_pipeline,
            current_chunk_x: self.current_chunk_x,
            current_chunk_z: self.current_chunk_z,
            chunk_batch_radius: self.chunk_batch_radius,
            loaded_chunks: self.loaded_chunks,
            chunk_sender: self.chunk_sender,
        }
    }

    fn position(self) -> PositionSessionContext<'a, 'b> {
        PositionSessionContext {
            world_root: self.world_root,
            world_seed: self.world_seed,
            profile_uuid: &self.profile.uuid,
            world_items: self.world_items,
            recipe_manager: self.recipe_manager,
            chunk_pipeline: self.chunk_pipeline,
            current_chunk_x: self.current_chunk_x,
            current_chunk_z: self.current_chunk_z,
            chunk_batch_radius: self.chunk_batch_radius,
            loaded_chunks: self.loaded_chunks,
            chunk_sender: self.chunk_sender,
        }
    }

    fn use_item_on<'c>(self) -> UseItemOnContext<'c, 'c>
    where
        'a: 'c,
        'b: 'c,
    {
        UseItemOnContext {
            world_layout: self.world_layout,
            world_seed: self.world_seed,
            chunk_cache: self.chunk_cache,
            live_fluid_ticks: self.live_fluid_ticks,
            live_block_ticks: self.live_block_ticks,
            game_time: self.play_tick_count as i64,
            player_access: self.player_access,
            profile_uuid: &self.profile.uuid,
            spawn_protection_radius: self.properties.spawn_protection,
        }
    }

    fn action(self) -> PlayerActionContext<'a, 'b> {
        PlayerActionContext {
            world_root: self.world_root,
            world_seed: self.world_seed,
            world_layout: self.world_layout,
            chunk_cache: self.chunk_cache,
            live_fluid_ticks: self.live_fluid_ticks,
            live_block_ticks: self.live_block_ticks,
            play_tick_count: self.play_tick_count,
            world_items: self.world_items,
            player_access: self.player_access,
            profile_uuid: &self.profile.uuid,
            spawn_protection_radius: self.properties.spawn_protection,
        }
    }

    fn inventory(&self) -> InventoryPacketContext<'a, 'b> {
        InventoryPacketContext {
            recipe_manager: self.recipe_manager,
            world_layout: self.world_layout,
            world_seed: self.world_seed,
            chunk_cache: self.chunk_cache,
            profile_name: &self.profile.name,
        }
    }
}

fn write_unexpected_play_packet_disconnect(
    stream: &mut TcpStream,
    compression: CompressionState,
    packet_id: i32,
) -> io::Result<()> {
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
    )
}

fn handle_login_connection(
    stream: &mut TcpStream,
    mut context: LoginConnectionContext<'_>,
) -> io::Result<()> {
    let LoginHandshakeOutcome::Complete(CompletedLogin {
        finished,
        compression,
        // Bound (not `_`) so it lives for the whole play session below and only
        // deregisters the player from the ActiveLoginRegistry on disconnect.
        active_login,
    }) = complete_login_handshake(stream, &mut context)?
    else {
        return Ok(());
    };
    run_configuration_handshake(
        stream,
        context.shared.properties,
        compression,
        context.rate_limiter,
        &active_login,
    )?;
    // The player now enters the PLAY state — Java `PlayerList.placeNewPlayer`
    // adds them to `getPlayers()`, which is what the status online count reflects.
    active_login.mark_in_play();
    let LoginConnectionContext {
        shared,
        remote_address,
        rate_limiter,
        ..
    } = context;
    run_joined_play_session(
        stream,
        compression,
        finished,
        PlayConnectionContext {
            shared,
            remote_address,
            rate_limiter,
            active_login: &active_login,
        },
    )
}

// The play session is a flat orchestrator: it unpacks the joined-session state
// and then loops, delegating each tick to `tick_joined_play_session_loop` and
// each inbound packet to `read_and_dispatch_joined_play_packet`. Its length is
// almost entirely per-field context plumbing, so it reads more clearly as one
// function than split across artificial seams.
#[allow(clippy::too_many_lines)]
fn run_joined_play_session(
    stream: &mut TcpStream,
    compression: CompressionState,
    finished: ClientboundLoginFinishedPacket,
    context: PlayConnectionContext<'_>,
) -> io::Result<()> {
    let PlayConnectionContext {
        shared,
        remote_address,
        rate_limiter,
        active_login,
    } = context;
    let ConnectionSharedContext {
        properties,
        favicon: _,
        chunk_cache,
        chunk_pipeline,
        player_access,
        world_root,
        world_seed,
        clock,
        weather,
        recipe_manager,
        world_items,
        world_mobs,
        ..
    } = shared;
    let JoinedPlaySessionStart {
        mut play_state,
        mut current_chunk_x,
        mut current_chunk_z,
        chunk_batch_radius,
        mut loaded_chunks,
        mut chunk_sender,
        mut chunk_pipeline_stats,
        mut keep_alive,
        keep_alive_epoch,
        mut last_sent_rain_level,
        mut last_sent_thunder_level,
        mut last_time_sync,
        mut live_block_ticks,
        world_layout,
        mut last_item_tick,
        mut last_player_tick,
        mut play_tick_count,
        mut live_fluid_ticks,
    } = initialize_joined_play_session(stream, compression, &finished, shared, remote_address)?;
    // The per-session `chunk_sender` (Java `PlayerChunkSender`), weather-level
    // latches, and item-entity tick timer were all seeded in
    // `initialize_joined_play_session`; the per-tick loop below drains/advances
    // them. `join_commands_sent` latches the one-shot `/biome` command tree, sent
    // once after the first chunk batch.
    let mut join_commands_sent = false;
    loop {
        if !tick_joined_play_session_loop(
            stream,
            compression,
            &mut play_state,
            JoinedPlayLoopTickContext {
                properties,
                world_root,
                world_seed,
                clock,
                weather,
                world_items,
                world_mobs,
                chunk_cache,
                chunk_pipeline,
                current_chunk_x,
                current_chunk_z,
                loaded_chunks: &loaded_chunks,
                chunk_sender: &mut chunk_sender,
                chunk_pipeline_stats: &mut chunk_pipeline_stats,
                live_fluid_ticks: &mut live_fluid_ticks,
                live_block_ticks: &mut live_block_ticks,
                world_layout: &world_layout,
                keep_alive: &mut keep_alive,
                keep_alive_epoch,
                last_time_sync: &mut last_time_sync,
                last_item_tick: &mut last_item_tick,
                last_player_tick: &mut last_player_tick,
                play_tick_count: &mut play_tick_count,
                last_sent_rain_level: &mut last_sent_rain_level,
                last_sent_thunder_level: &mut last_sent_thunder_level,
                join_commands_sent: &mut join_commands_sent,
            },
        )? {
            // Keepalive timed out (disconnect.timeout already written).
            return Ok(());
        }

        let packet_outcome = read_and_dispatch_joined_play_packet(
            stream,
            compression,
            rate_limiter,
            &mut play_state,
            JoinedPlayPacketStepContext {
                properties,
                player_access,
                world_root,
                world_seed,
                profile: &finished.profile,
                recipe_manager,
                world_layout: &world_layout,
                chunk_cache,
                chunk_pipeline,
                world_items,
                world_mobs,
                weather,
                current_chunk_x: &mut current_chunk_x,
                current_chunk_z: &mut current_chunk_z,
                chunk_batch_radius,
                loaded_chunks: &mut loaded_chunks,
                chunk_sender: &mut chunk_sender,
                live_fluid_ticks: &mut live_fluid_ticks,
                live_block_ticks: &mut live_block_ticks,
                play_tick_count,
                active_login,
                keep_alive: &mut keep_alive,
                keep_alive_epoch,
            },
        )?;
        if let PlayPacketDispatchOutcome::EndSession = packet_outcome {
            return Ok(());
        }
    }
}

pub(super) fn read_expected_login_hello_packet<R: Read>(
    reader: &mut R,
    rate_limiter: &mut PacketRateLimiter,
) -> io::Result<ServerboundHelloPacket> {
    let packet = read_packet_with_rate_limit(reader, CompressionState::disabled(), rate_limiter)?;
    let mut input = Cursor::new(packet);
    let packet_id = read_var_i32(&mut input)?;
    if packet_id != SERVERBOUND_HELLO_PACKET_ID {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected login hello",
        ));
    }
    ServerboundHelloPacket::read(&mut input)
}

#[cfg(test)]
mod resource_usage_tests {
    use super::*;

    #[test]
    fn process_cpu_jiffies_parser_handles_process_names_with_spaces() {
        let stat = "123 (rust craft server) S 1 2 3 4 5 6 7 8 9 10 123 45 14 15";

        assert_eq!(read_process_cpu_jiffies(stat), Some(168));
    }

    #[test]
    fn total_cpu_jiffies_parser_sums_aggregate_cpu_line() {
        let stat = "cpu  100 20 30 400 5 6 7 8 9 10\ncpu0 1 2 3 4";

        assert_eq!(read_total_cpu_jiffies(stat), Some(595));
    }

    #[test]
    fn resident_memory_parser_reads_vmrss_kib() {
        let status = "Name:\tvibecraft\nVmPeak:\t2048 kB\nVmRSS:\t1536 kB\n";

        assert_eq!(parse_resident_memory_kib(status), Some(1536));
    }
}

#[cfg(test)]
mod spawn_protection_wiring_tests {
    use super::*;
    use crate::block_update::BlockPos;
    use crate::player_access::{NameAndId, OpEntry, PlayerAccess};
    use crate::storage::nbt::Tag;

    /// `build_limit_message_tag` must match Java `ServerPlayer.sendBuildLimitMessage`:
    /// translatable `build.tooHigh`/`build.tooLow` with the integer limit arg, RED.
    #[test]
    fn build_limit_message_tag_matches_vanilla_translatable_component() {
        let high = build_limit_message_tag(true, 319);
        let Tag::Compound(fields) = high else {
            panic!("expected compound");
        };
        let get = |k: &str| fields.iter().find(|(n, _)| n == k).map(|(_, v)| v);
        assert_eq!(
            get("translate"),
            Some(&Tag::String("build.tooHigh".to_string()))
        );
        assert_eq!(get("with"), Some(&Tag::List(vec![Tag::Int(319)])));
        assert_eq!(get("color"), Some(&Tag::String("red".to_string())));

        let low = build_limit_message_tag(false, -64);
        let Tag::Compound(fields) = low else {
            panic!("expected compound");
        };
        assert_eq!(
            fields
                .iter()
                .find(|(n, _)| n == "translate")
                .map(|(_, v)| v),
            Some(&Tag::String("build.tooLow".to_string()))
        );
    }

    /// Java `Player.blockActionRestricted`: spectators can never break blocks;
    /// survival/creative/adventure are not categorically restricted here.
    #[test]
    fn spectators_cannot_break_blocks() {
        assert!(spectator_cannot_break(GameMode::Spectator));
        assert!(!spectator_cannot_break(GameMode::Survival));
        assert!(!spectator_cannot_break(GameMode::Creative));
        assert!(!spectator_cannot_break(GameMode::Adventure));
    }

    /// Java handleBlockBreakAction rejects `pos.getY() > getMaxY()`; the overworld
    /// ceiling is `OVERWORLD_MIN_Y + OVERWORLD_LEVEL_HEIGHT - 1` = 319.
    #[test]
    fn block_break_rejected_above_overworld_build_height() {
        assert!(!block_break_above_build_height(319));
        assert!(!block_break_above_build_height(0));
        assert!(!block_break_above_build_height(-64));
        assert!(block_break_above_build_height(320));
        assert!(block_break_above_build_height(1000));
    }

    #[test]
    fn live_block_break_drop_gate_uses_held_tool_like_java() {
        let hand = ItemStack::empty();
        assert_eq!(should_drop_block_loot("minecraft:stone", Some(&hand)).0, false);

        let wooden_pick = ItemStack::new("minecraft:wooden_pickaxe", 1);
        assert!(should_drop_block_loot("minecraft:stone", Some(&wooden_pick)).0);
        assert!(!should_drop_block_loot("minecraft:iron_ore", Some(&wooden_pick)).0);

        let diamond_pick = ItemStack::new("minecraft:diamond_pickaxe", 1);
        assert!(should_drop_block_loot("minecraft:iron_ore", Some(&diamond_pick)).0);

        let apple = ItemStack::new("minecraft:apple", 1);
        assert!(should_drop_block_loot("minecraft:dirt", Some(&apple)).0);
    }

    #[test]
    fn survival_block_break_damages_main_hand_tool_like_java_mine_block() {
        let mut state = PlaySessionState::default();
        state.selected_slot = 0;
        state
            .inventory_menu
            .player_inventory_mut()
            .set(0, ItemStack::new("minecraft:diamond_sword", 1));

        assert!(damage_main_hand_tool_after_block_break(
            &mut state,
            Some("minecraft:cobweb")
        ));
        assert_eq!(
            state
                .inventory_menu
                .player_inventory()
                .get(0)
                .damage_value(),
            2
        );
        assert!(!damage_main_hand_tool_after_block_break(
            &mut state,
            Some("minecraft:short_grass")
        ));
        assert_eq!(
            state
                .inventory_menu
                .player_inventory()
                .get(0)
                .damage_value(),
            2
        );
    }

    #[test]
    fn live_destroy_tick_uses_selected_tool_speed_for_progress() {
        let physics = crate::block_properties::state_physics_by_name("minecraft:stone");
        let mut state = PlaySessionState::default();
        state.selected_slot = 0;
        state.block_break_state.is_destroying = true;
        state.block_break_state.destroy_progress_start = 0;
        state.block_break_state.game_ticks = 5;

        let (hand_speed, hand_correct_tool) =
            live_block_destroy_tool_inputs(&state, "minecraft:stone");
        assert_eq!(
            block_destroy_progress_state(
                &state.block_break_state,
                physics,
                hand_speed,
                hand_correct_tool,
                0
            ),
            Some(0)
        );

        state
            .inventory_menu
            .player_inventory_mut()
            .set(0, ItemStack::new("minecraft:diamond_pickaxe", 1));
        let (pick_speed, pick_correct_tool) =
            live_block_destroy_tool_inputs(&state, "minecraft:stone");
        assert_eq!(pick_speed, 8.0);
        assert!(pick_correct_tool);
        assert_eq!(
            block_destroy_progress_state(
                &state.block_break_state,
                physics,
                pick_speed,
                pick_correct_tool,
                0
            ),
            Some(10)
        );
    }

    #[test]
    fn block_destruction_progress_is_not_echoed_to_breaking_player() {
        let mut bytes = Vec::new();
        write_block_destruction(
            &mut bytes,
            CompressionState::disabled(),
            crate::block_update::BlockPos { x: 1, y: 64, z: 1 },
            5,
        )
        .unwrap();
        assert!(
            bytes.is_empty(),
            "Java ServerLevel.destroyBlockProgress excludes the breaking player's own connection"
        );
    }

    /// The spawn-protection overlay component must match Java
    /// `Component.translatable("build.spawn_protection", pos.toShortString())`
    /// with RED color: a compound with `translate`/`with`/`color`, where the
    /// single `with` arg is the `"x, y, z"` short-string of the block pos.
    #[test]
    fn spawn_protection_message_tag_matches_vanilla_translatable_component() {
        let tag = spawn_protection_message_tag(10, 64, -3);
        let Tag::Compound(fields) = tag else {
            panic!("expected compound component");
        };
        let get = |key: &str| fields.iter().find(|(k, _)| k == key).map(|(_, v)| v);
        assert_eq!(
            get("translate"),
            Some(&Tag::String("build.spawn_protection".to_string()))
        );
        assert_eq!(get("color"), Some(&Tag::String("red".to_string())));
        assert_eq!(
            get("with"),
            Some(&Tag::List(vec![Tag::String("10, 64, -3".to_string())]))
        );
    }

    fn op_access() -> (PlayerAccess, String) {
        let mut access = PlayerAccess::default();
        let op = NameAndId {
            uuid: "00000000-0000-0000-0000-000000000002".to_string(),
            name: "Alex".to_string(),
        };
        access.op(OpEntry {
            user: op.clone(),
            level: 4,
            bypasses_player_limit: true,
        });
        (access, op.uuid)
    }

    /// The live block-break gate (`spawn_protection_break_denied`) must mirror
    /// Java `DedicatedServer.isUnderSpawnProtection`: a non-op breaking inside
    /// the spawn radius on a server that has ops is denied; ops, breaks outside
    /// the radius, a zero radius, and ops-less servers are all allowed.
    #[test]
    fn live_break_denies_nonop_in_spawn_radius_and_allows_op_or_outside() {
        let (access, op_uuid) = op_access();
        let nonop = "00000000-0000-0000-0000-000000000009";
        let spawn = BlockPos { x: 0, y: 64, z: 0 };

        // Non-op inside the chebyshev radius -> denied.
        assert!(spawn_protection_break_denied(
            &access,
            16,
            spawn,
            nonop,
            BlockPos { x: 10, y: 64, z: 0 },
        ));
        // Operator inside the radius -> allowed.
        assert!(!spawn_protection_break_denied(
            &access,
            16,
            spawn,
            &op_uuid,
            BlockPos { x: 10, y: 64, z: 0 },
        ));
        // Non-op just outside the radius (dist 17 > 16) -> allowed.
        assert!(!spawn_protection_break_denied(
            &access,
            16,
            spawn,
            nonop,
            BlockPos { x: 17, y: 64, z: 0 },
        ));
        // spawn-protection=0 disables the check entirely.
        assert!(!spawn_protection_break_denied(
            &access,
            0,
            spawn,
            nonop,
            BlockPos { x: 0, y: 64, z: 0 },
        ));
        // A server with no operators never protects spawn (Java: getOps().isEmpty()).
        assert!(!spawn_protection_break_denied(
            &PlayerAccess::default(),
            16,
            spawn,
            nonop,
            BlockPos { x: 0, y: 64, z: 0 },
        ));
    }
}
