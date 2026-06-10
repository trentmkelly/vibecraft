use super::*;

impl Default for PlaySessionState {
    fn default() -> Self {
        Self {
            x: 0.5,
            y: SPAWN_Y,
            z: 0.5,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: true,
            fall_distance: 0.0,
            selected_slot: 0,
            health: 20.0,
            food_level: 20,
            food_saturation: 5.0,
            food_exhaustion: 0.0,
            food_tick_timer: 0,
            input_forward: false,
            input_backward: false,
            input_left: false,
            input_right: false,
            input_shift: false,
            input_sprinting: false,
            input_jumping: false,
            air_supply: MAX_AIR_SUPPLY,
            in_water: false,
            eye_in_water: false,
            water_fluid_height: 0.0,
            water_velocity_x: 0.0,
            water_velocity_y: 0.0,
            water_velocity_z: 0.0,
            xp_progress: 0.0,
            xp_level: 0,
            xp_total: 0,
            xp_seed: 0,
            score: 0,
            game_mode: GameMode::Survival,
            previous_game_mode: None,
            spawn: None,
            seen_credits: false,
            entered_nether_position: None,
            last_death_location: None,
            root_vehicle: None,
            active_effects: Vec::new(),
            ender_items: Vec::new(),
            abilities: PlayerNbtAbilities::default_survival(),
            inventory_menu: InventoryMenu::new(PlayerInventory::new(), RecipeMap::default()),
            carried_item: ItemStack::empty(),
            container_state_id: 0,
            next_container_id: 1,
            recipe_book_settings: ClientboundRecipeBookSettingsPacket {
                crafting: RecipeBookTypeSettings::CLOSED_UNFILTERED,
                furnace: RecipeBookTypeSettings::CLOSED_UNFILTERED,
                blast_furnace: RecipeBookTypeSettings::CLOSED_UNFILTERED,
                smoker: RecipeBookTypeSettings::CLOSED_UNFILTERED,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerSpawnData {
    pub dimension: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub forced: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerGlobalPosData {
    pub dimension: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerSpawnPlacement {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerNbtAbilities {
    pub invulnerable: bool,
    pub flying: bool,
    pub mayfly: bool,
    pub instabuild: bool,
    pub may_build: bool,
    pub fly_speed: f32,
    pub walk_speed: f32,
}

impl PlayerNbtAbilities {
    pub fn default_survival() -> Self {
        Self {
            invulnerable: false,
            flying: false,
            mayfly: false,
            instabuild: false,
            may_build: true,
            fly_speed: 0.05,
            walk_speed: 0.1,
        }
    }

    pub fn for_game_mode(game_mode: GameMode) -> Self {
        let mut abilities = Self::default_survival();
        abilities.apply_game_mode(game_mode);
        abilities
    }

    pub fn apply_game_mode(&mut self, game_mode: GameMode) {
        // Java: GameType.updatePlayerAbilities mutates the permission flags
        // after loading player NBT and whenever the server changes gamemode.
        match game_mode {
            GameMode::Creative => {
                self.mayfly = true;
                self.instabuild = true;
                self.invulnerable = true;
            }
            GameMode::Spectator => {
                self.mayfly = true;
                self.instabuild = false;
                self.invulnerable = true;
                self.flying = true;
            }
            GameMode::Survival | GameMode::Adventure => {
                self.mayfly = false;
                self.instabuild = false;
                self.invulnerable = false;
                self.flying = false;
            }
        }
        // Java: abilities.mayBuild = !this.isBlockPlacingRestricted()
        // isBlockPlacingRestricted returns true for ADVENTURE and SPECTATOR.
        self.may_build = !matches!(game_mode, GameMode::Adventure | GameMode::Spectator);
    }
}
#[allow(dead_code)]
pub const SPAWN_CHUNK_SECTION_COUNT: usize = 24;
pub const AIR_BLOCK_STATE_ID: i32 = 0;
pub const STONE_BLOCK_STATE_ID: i32 = 1;
pub const GRANITE_BLOCK_STATE_ID: i32 = 2;
pub const DIORITE_BLOCK_STATE_ID: i32 = 4;
pub const ANDESITE_BLOCK_STATE_ID: i32 = 6;
pub const GRASS_BLOCK_STATE_ID: i32 = 9;
pub const DIRT_BLOCK_STATE_ID: i32 = 10;
pub const BEDROCK_BLOCK_STATE_ID: i32 = 85;
pub const SHORT_GRASS_BLOCK_STATE_ID: i32 = 131;
pub const DANDELION_BLOCK_STATE_ID: i32 = 158;
pub const POPPY_BLOCK_STATE_ID: i32 = 161;
#[allow(dead_code)]
pub const PLAINS_BIOME_ID: i32 = 40;
pub const TERRAIN_BASE_Y: i32 = 64;
pub const TERRAIN_MIN_SURFACE_Y: i32 = 70;
pub const ITEM_ENTITY_TYPE_ID: i32 = 71;
pub const SPAWN_Y: f64 = 112.0;
pub const PLAYER_ENTITY_ID: i32 = 1;
pub const PLAYER_WIDTH: f64 = 0.6;
pub const PLAYER_HEIGHT: f64 = 1.8;
pub const PLAYER_EYE_HEIGHT: f64 = 1.62;
pub const MAX_AIR_SUPPLY: i32 = 300;
pub const DROWN_AIR_SUPPLY_THRESHOLD: i32 = -20;
pub const DROWN_DAMAGE: f32 = 2.0;
pub const WATER_MOVE_RELATIVE_SPEED: f64 = 0.02;
pub const WATER_HORIZONTAL_SLOWDOWN: f64 = 0.8;
pub const WATER_SPRINTING_HORIZONTAL_SLOWDOWN: f64 = 0.9;
pub const WATER_VERTICAL_SLOWDOWN: f64 = 0.8;
pub const WATER_FALLING_GRAVITY: f64 = 0.005;
pub const WATER_JUMP_IMPULSE: f64 = 0.04;
pub const REGION_FEATURE_GENERATION_RADIUS: i32 = 3;
pub const REGION_FEATURE_CACHEABLE_RADIUS: i32 = 1;

#[derive(Clone, Default)]
pub struct GeneratedChunkCache {
    pub chunks: Arc<Mutex<HashMap<ChunkPos, Arc<LevelChunk>>>>,
    /// Chunks that have been mutated in memory since they were last persisted
    /// to disk. Java mirror: `LevelChunk.unsaved` flag — set on every
    /// `setBlockState` and cleared by `ChunkMap.save`.
    ///
    /// The set is drained by [`Self::flush_dirty`], which runs on a periodic
    /// background timer and at every play-session exit. Keeping the dirty
    /// set in the cache (instead of per-session) means two players editing
    /// the same chunk both contribute to the same flush.
    pub dirty: Arc<Mutex<HashSet<ChunkPos>>>,
}

fn lock_mutex<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

impl GeneratedChunkCache {
    pub fn get_or_load(
        &self,
        x: i32,
        z: i32,
        world_root: &Path,
        world_seed: i64,
    ) -> Arc<LevelChunk> {
        let pos = ChunkPos { x, z };
        if let Some(chunk) = lock_mutex(&self.chunks).get(&pos).cloned() {
            return chunk;
        }

        if live_region_feature_generation_enabled()
            && live_chunk_generation_mode() == LiveChunkGenerationMode::RealSurface
        {
            let region_dir = world_root.join("region");
            if try_load_chunk_from_region(&region_dir, pos).is_none() {
                match generate_overworld_spawn_chunk_region_for_preset_with_mode(
                    pos,
                    REGION_FEATURE_GENERATION_RADIUS,
                    "normal",
                    LiveChunkGenerationMode::RealSurface,
                    world_seed,
                    true,
                ) {
                    Ok(region_chunks) => {
                        let mut cache = lock_mutex(&self.chunks);
                        for (region_pos, chunk) in region_chunks {
                            if !region_generated_chunk_is_cacheable(pos, region_pos) {
                                continue;
                            }
                            cache.entry(region_pos).or_insert_with(|| Arc::new(chunk));
                        }
                        if let Some(chunk) = cache.get(&pos).cloned() {
                            return chunk;
                        }
                    }
                    Err(err) => {
                        eprintln!(
                            "[worldgen-region] center=({}, {}) failed: {}",
                            pos.x, pos.z, err
                        );
                    }
                }
            }
        }

        let chunk = Arc::new(load_or_generate_spawn_chunk_uncached(
            x, z, world_root, world_seed,
        ));
        lock_mutex(&self.chunks)
            .entry(pos)
            .or_insert_with(|| Arc::clone(&chunk))
            .clone()
    }

    /// Evict a chunk from the in-memory cache. **Refuses to evict dirty
    /// chunks**: an in-memory-only block change would be silently lost if
    /// we dropped it before `flush_dirty` ran. Java's chunk map has the
    /// same invariant — `LevelChunk.unsaved` blocks unload until the
    /// chunk has been persisted.
    #[cfg(test)]
    pub fn invalidate(&self, pos: ChunkPos) {
        if lock_mutex(&self.dirty).contains(&pos) {
            return;
        }
        lock_mutex(&self.chunks).remove(&pos);
    }

    /// Nonblocking readiness probe.
    ///
    /// Returns `Some(chunk)` only if the chunk has already been generated and
    /// is sitting in the cache; never triggers generation or I/O. Mirrors
    /// Java `ChunkMap.getChunkToSend`: the per-tick chunk send path calls this
    /// to decide which pending chunks can be flushed *right now*, and skips
    /// the rest for a later tick once generation completes.
    pub fn try_get_ready(&self, pos: ChunkPos) -> Option<Arc<LevelChunk>> {
        lock_mutex(&self.chunks).get(&pos).cloned()
    }

    /// Java mirror: `LevelChunk.setBlockState(pos, state, flags)`. Mutates
    /// the in-memory chunk and marks it dirty for later persistence.
    /// Returns the previous block name (filtered to drop air), matching
    /// what the old `break_block_in_region` returned for the loot path.
    ///
    /// If the chunk isn't yet cached, it's loaded first via [`get_or_load`]
    /// (so block updates from worker/player paths transparently bring the
    /// chunk into the cache). The mutation goes through `Arc::make_mut`,
    /// which clones the inner `LevelChunk` only if other holders (e.g.
    /// in-flight chunk send batches) still reference the previous
    /// snapshot — those readers are not affected by the mutation, which
    /// matches Java's invariant that send packets capture chunk state at
    /// packet-build time.
    ///
    /// Avoiding the previous write-through-to-disk pattern is what fixes
    /// the multi-tens-of-ms `[fluid-timing] write_update` cost: flowing
    /// fluids used to re-encode and write a full 24-section chunk NBT to
    /// the region file on every tick.
    pub fn set_block(
        &self,
        world_root: &Path,
        world_seed: i64,
        pos: crate::block_update::BlockPos,
        block_name: &str,
    ) -> Option<String> {
        let chunk_pos = ChunkPos {
            x: pos.x.div_euclid(16),
            z: pos.z.div_euclid(16),
        };
        // Ensure the chunk is in cache. get_or_load handles
        // region-read + worldgen fallback.
        let _ = self.get_or_load(chunk_pos.x, chunk_pos.z, world_root, world_seed);
        let prev = {
            let mut map = lock_mutex(&self.chunks);
            let arc = map.get_mut(&chunk_pos)?;
            let chunk = Arc::make_mut(arc);
            let prev = chunk
                .get_block_state(pos.x, pos.y, pos.z)
                .filter(|n| n != "minecraft:air");
            chunk.set_block_state(pos.x, pos.y, pos.z, block_name);
            prev
        };
        lock_mutex(&self.dirty).insert(chunk_pos);
        prev
    }

    /// Persist every dirty chunk to its region file and clear the dirty
    /// set. Returns the number of chunks written.
    ///
    /// Java mirror: `ChunkMap.processUnloads` + `ChunkHolder.save` — the
    /// periodic chunk-save pass that runs ~every autosave interval and on
    /// shutdown. We run it on a background timer (see
    /// `spawn_chunk_flush_thread`) and at every play-session exit so a
    /// disconnect within the autosave window doesn't drop the player's
    /// edits.
    pub fn flush_dirty(
        &self,
        world_root: &Path,
        sync_chunk_writes: bool,
        region_file_compression: RegionCompression,
    ) -> usize {
        let dirty: Vec<ChunkPos> = {
            let mut d = lock_mutex(&self.dirty);
            d.drain().collect()
        };
        if dirty.is_empty() {
            return 0;
        }
        let snapshots: Vec<(ChunkPos, Arc<LevelChunk>)> = {
            let map = lock_mutex(&self.chunks);
            dirty
                .iter()
                .filter_map(|pos| map.get(pos).cloned().map(|c| (*pos, c)))
                .collect()
        };
        let region_dir = world_root.join("region");
        let mut written = 0_usize;
        for (pos, chunk) in snapshots {
            let Ok(region) = RegionFile::open_with_options(
                &region_dir,
                pos.region(),
                sync_chunk_writes,
                region_file_compression,
            ) else {
                continue;
            };
            let nbt = chunk.to_nbt(crate::storage::datafix::TARGET_DATA_VERSION);
            if region.write_chunk_nbt(pos, "", &nbt).is_ok() {
                written += 1;
            }
        }
        written
    }
}

/// Spawn a background thread that periodically flushes dirty chunks from
/// `cache` to the region files under `world_root`. Java mirror: the
/// autosave loop inside `MinecraftServer.tickServer` that calls
/// `ChunkMap.processUnloads` and triggers `ChunkHolder.save` for chunks
/// whose `unsaved` flag is set.
///
/// Runs forever once started; the thread is daemon-style (the OS reaps
/// it at process exit) and never holds locks across writes.
pub fn spawn_chunk_flush_thread(
    cache: GeneratedChunkCache,
    world_root: Arc<PathBuf>,
    interval: Duration,
    sync_chunk_writes: bool,
    region_file_compression: RegionCompression,
) {
    if let Err(err) = thread::Builder::new()
        .name("chunk-flush".to_string())
        .spawn(move || loop {
            thread::sleep(interval);
            let started = Instant::now();
            let written =
                cache.flush_dirty(&world_root, sync_chunk_writes, region_file_compression);
            if written > 0 {
                eprintln!(
                    "[chunk-flush] persisted {} dirty chunks in {}ms",
                    written,
                    started.elapsed().as_millis()
                );
            }
        })
    {
        eprintln!("[chunk-flush] failed to spawn background chunk flush thread: {err}");
    }
}

/// Shared async chunk generation coordinator.
///
/// Java split: `ChunkMap` + `ChunkTaskDispatcher` schedule generation on
/// background executors while the server tick loop stays responsive. Here we
/// keep [`GeneratedChunkCache`] as the read-only cache of completed chunks
/// and layer this struct on top to own the worker pool and the
/// "currently-pending" registry.
///
/// One instance is constructed per running server and shared across all play
/// sessions, so two players viewing the same chunk coalesce into a single
/// generation job. Completed chunks remain visible via
/// [`GeneratedChunkCache::try_get_ready`].
#[derive(Clone)]
pub struct ChunkPipeline {
    pub cache: GeneratedChunkCache,
    pub inner: Arc<ChunkPipelineInner>,
}

pub struct ChunkPipelineInner {
    pub world_root: PathBuf,
    pub world_seed: i64,
    pub state: Mutex<ChunkPipelineState>,
    pub cvar: Condvar,
    pub shutdown: AtomicBool,
    /// Generations completed since startup, for diagnostics.
    pub generated_total: AtomicU64,
}

#[derive(Default)]
pub struct ChunkPipelineState {
    /// Positions with outstanding work — queued or in flight.
    ///
    /// Each entry records when the request was first enqueued, so the
    /// diagnostics layer can report the oldest pending age (a proxy for
    /// "worker pool is overloaded").
    pub pending: HashMap<ChunkPos, Instant>,
    /// FIFO of positions awaiting a worker. Drained from the front by
    /// workers, refilled by [`ChunkPipeline::request_chunk`].
    pub queue: VecDeque<ChunkPos>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ChunkPipelineDiagnostics {
    pub queue_depth: usize,
    pub in_flight: usize,
    pub oldest_request_age_ms: u64,
    pub generated_total: u64,
}

impl ChunkPipeline {
    /// Create the pipeline, spawning `worker_count` background generation
    /// workers. Workers block on a condvar when idle.
    pub fn new(
        cache: GeneratedChunkCache,
        world_root: PathBuf,
        world_seed: i64,
        worker_count: usize,
    ) -> Self {
        let inner = Arc::new(ChunkPipelineInner {
            world_root,
            world_seed,
            state: Mutex::new(ChunkPipelineState::default()),
            cvar: Condvar::new(),
            shutdown: AtomicBool::new(false),
            generated_total: AtomicU64::new(0),
        });
        for worker_id in 0..worker_count {
            let inner = Arc::clone(&inner);
            let cache = cache.clone();
            if let Err(err) = thread::Builder::new()
                .name(format!("chunk-pipeline-{worker_id}"))
                .spawn(move || chunk_pipeline_worker(inner, cache))
            {
                eprintln!("[chunk-pipeline] failed to spawn worker {worker_id}: {err}");
            }
        }
        Self { cache, inner }
    }

    /// Schedule generation for `pos` if it isn't already ready or in flight.
    ///
    /// Idempotent: duplicate calls (across sessions or after redraws) coalesce
    /// into the same generation job. Already-cached chunks are a no-op.
    pub fn request_chunk(&self, pos: ChunkPos) {
        if self.cache.try_get_ready(pos).is_some() {
            return;
        }
        let mut state = lock_mutex(&self.inner.state);
        if state.pending.contains_key(&pos) {
            return;
        }
        state.pending.insert(pos, Instant::now());
        state.queue.push_back(pos);
        self.inner.cvar.notify_one();
    }

    /// Nonblocking readiness lookup, delegating to the cache. The pipeline
    /// also accepts chunks that arrived via any other path (region load,
    /// direct synchronous spawn) — readiness is purely a function of the
    /// cache state.
    pub fn try_get_ready(&self, pos: ChunkPos) -> Option<Arc<LevelChunk>> {
        self.cache.try_get_ready(pos)
    }

    /// Best-effort cancel: pull `pos` out of the queue if it hasn't started
    /// yet. Workers cannot be preempted mid-generation, so an in-flight job
    /// runs to completion (the chunk lands in the cache and is available if
    /// the player re-enters its tracking range later).
    pub fn cancel_request(&self, pos: ChunkPos) {
        let mut state = lock_mutex(&self.inner.state);
        if state.pending.contains_key(&pos) {
            let before = state.queue.len();
            state.queue.retain(|p| *p != pos);
            if state.queue.len() < before {
                // Only fully drop the pending record if we actually pulled the
                // job from the queue. In-flight jobs keep the pending entry so
                // we don't accidentally schedule the same chunk twice while a
                // worker is still computing it.
                state.pending.remove(&pos);
            }
        }
    }

    pub fn diagnostics(&self) -> ChunkPipelineDiagnostics {
        let state = lock_mutex(&self.inner.state);
        let now = Instant::now();
        let oldest = state
            .pending
            .values()
            .map(|t| now.saturating_duration_since(*t))
            .max();
        ChunkPipelineDiagnostics {
            queue_depth: state.queue.len(),
            in_flight: state.pending.len() - state.queue.len(),
            oldest_request_age_ms: oldest.map(|d| d.as_millis() as u64).unwrap_or(0),
            generated_total: self.inner.generated_total.load(Ordering::Relaxed),
        }
    }

    /// Underlying cache, for paths that still need direct read access (region
    /// load, invalidation on block updates, fluid seeding).
    pub fn cache(&self) -> &GeneratedChunkCache {
        &self.cache
    }
}

pub fn chunk_pipeline_worker(inner: Arc<ChunkPipelineInner>, cache: GeneratedChunkCache) {
    loop {
        let pos = {
            let mut state = lock_mutex(&inner.state);
            loop {
                if inner.shutdown.load(Ordering::Acquire) {
                    return;
                }
                if let Some(pos) = state.queue.pop_front() {
                    break pos;
                }
                state = match inner.cvar.wait(state) {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
            }
        };

        // Drop the pending record regardless of how generation exits — a
        // worker panic must not "lose" a chunk position forever, otherwise
        // future requests for the same pos would be silently coalesced away.
        struct PendingGuard<'a> {
            inner: &'a ChunkPipelineInner,
            pos: ChunkPos,
        }
        impl<'a> Drop for PendingGuard<'a> {
            fn drop(&mut self) {
                lock_mutex(&self.inner.state).pending.remove(&self.pos);
            }
        }
        let _guard = PendingGuard { inner: &inner, pos };

        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            cache.get_or_load(pos.x, pos.z, &inner.world_root, inner.world_seed)
        }));
        match result {
            Ok(_) => {
                inner.generated_total.fetch_add(1, Ordering::Relaxed);
            }
            Err(payload) => {
                let detail = panic_payload_to_string(&payload);
                eprintln!(
                    "[chunk-pipeline] worker panic generating chunk=({}, {}): {}",
                    pos.x, pos.z, detail
                );
            }
        }
    }
}

pub fn panic_payload_to_string(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&'static str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string panic payload>".to_string()
    }
}

#[derive(Debug, Clone)]
pub struct LiveFluidTicks {
    pub queues: LevelTickQueues,
}

impl LiveFluidTicks {
    pub fn new() -> Self {
        Self {
            queues: LevelTickQueues::new(),
        }
    }

    pub fn schedule(
        &mut self,
        game_time: i64,
        pos: crate::block_update::BlockPos,
        kind: FluidKind,
    ) {
        let chunk = ChunkPos {
            x: pos.x.div_euclid(16),
            z: pos.z.div_euclid(16),
        };
        self.queues.add_container(chunk);
        let tick = self.queues.create_tick(
            game_time,
            pos,
            kind.registry_id(),
            kind.tick_delay(),
            TickPriority::Normal,
        );
        let _ = self.queues.schedule(tick);
    }

    /// Java mirror: `LevelChunkTicks.unpack(currentTick)`. Schedule a tick at
    /// `current_tick + delay` with the saved priority — preserving exactly
    /// what was recorded in NBT instead of resetting the delay to the fluid's
    /// default. Used to restore the chunk's saved fluid ticks on load,
    /// equivalent to Java `ChunkAccess.unpackTicks` →
    /// `LevelChunk.registerTickContainerInLevel`.
    pub fn schedule_saved(
        &mut self,
        current_tick: i64,
        pos: crate::block_update::BlockPos,
        kind: FluidKind,
        delay: i32,
        priority: TickPriority,
    ) {
        let chunk = ChunkPos {
            x: pos.x.div_euclid(16),
            z: pos.z.div_euclid(16),
        };
        self.queues.add_container(chunk);
        let tick = self
            .queues
            .create_tick(current_tick, pos, kind.registry_id(), delay, priority);
        let _ = self.queues.schedule(tick);
    }

    pub fn tick_due(
        &mut self,
        game_time: i64,
        max_ticks: usize,
    ) -> Vec<crate::scheduled_tick::ScheduledTick> {
        self.queues.tick(game_time, max_ticks, |_| true)
    }
}

pub fn live_region_feature_generation_enabled() -> bool {
    matches!(
        std::env::var("VIBECRAFT_WORLDGEN_REGION_FEATURES").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

pub fn region_generated_chunk_is_cacheable(center: ChunkPos, candidate: ChunkPos) -> bool {
    (candidate.x - center.x).abs() <= REGION_FEATURE_CACHEABLE_RADIUS
        && (candidate.z - center.z).abs() <= REGION_FEATURE_CACHEABLE_RADIUS
}

#[derive(Clone, Default)]
pub struct ActiveLoginRegistry {
    pub sessions: Arc<Mutex<HashMap<String, ActiveLoginSession>>>,
    pub next_token: Arc<AtomicU64>,
}

pub struct ActiveLoginSession {
    pub token: u64,
    pub stream: TcpStream,
    /// Player profile name captured at login, used to build the status player
    /// sample (Java `NameAndId.name`).
    pub name: String,
    /// Whether the player has reached the PLAY state. Java only counts players in
    /// `PlayerList.getPlayers()` (added on play entry) toward the status online
    /// count, so login/configuration-phase sessions are excluded until this flips.
    pub in_play: bool,
    /// Whether the player opted into server listings (Java
    /// `ServerPlayer.allowsListing`). Defaults to `false` — matching
    /// `ClientInformation.createDefault()` — and is updated when a
    /// `ServerboundClientInformation` packet arrives during configuration or play.
    pub allows_listing: bool,
    /// The client's locale (Java `ClientInformation.language`), captured from the
    /// configuration-phase `ServerboundClientInformation`. Defaults to `en_us`
    /// (`ClientInformation.createDefault`). Used to pick the localized server code
    /// of conduct text (Java `ServerConfigurationPacketListenerImpl.addOptionalTasks`).
    pub language: String,
}

pub struct ActiveLoginGuard {
    pub sessions: Arc<Mutex<HashMap<String, ActiveLoginSession>>>,
    pub uuid: String,
    pub token: u64,
}

impl ActiveLoginGuard {
    /// Update this session's listing preference, 1:1 with the effect of Java
    /// `ServerPlayer.updateOptions` propagating `ClientInformation.allowsListing`.
    /// The token guard ensures a stale (already-replaced) login can never clobber
    /// the session that displaced it.
    pub fn set_allows_listing(&self, allows_listing: bool) {
        self.with_own_session(|session| session.allows_listing = allows_listing);
    }

    /// Mark this session as having entered the PLAY state so it is counted in the
    /// status online total (Java `PlayerList.placeNewPlayer` adding the player).
    pub fn mark_in_play(&self) {
        self.with_own_session(|session| session.in_play = true);
    }

    /// Capture the client's locale from its configuration-phase `ClientInformation`,
    /// stored lowercased (Java looks up `codeOfConducts.get(language.toLowerCase())`).
    pub fn set_language(&self, language: &str) {
        let language = language.to_lowercase();
        self.with_own_session(|session| session.language = language);
    }

    /// The client's locale, or `en_us` if no `ClientInformation` has arrived yet.
    pub fn language(&self) -> String {
        match self.sessions.lock() {
            Ok(sessions) => sessions
                .get(&self.uuid)
                .filter(|session| session.token == self.token)
                .map(|session| session.language.clone())
                .unwrap_or_else(|| "en_us".to_string()),
            Err(_) => "en_us".to_string(),
        }
    }

    /// Snapshot every player currently in the PLAY state as `NameAndId`, for
    /// commands that enumerate the online roster (Java `PlayerList.getPlayers`,
    /// e.g. `/list`). The guard shares the registry's session map, so this sees
    /// all sessions, not just its own.
    pub fn in_play_profiles(&self) -> Vec<NameAndId> {
        match self.sessions.lock() {
            Ok(sessions) => sessions
                .iter()
                .filter(|(_, session)| session.in_play)
                .map(|(uuid, session)| NameAndId {
                    uuid: uuid.clone(),
                    name: session.name.clone(),
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    fn with_own_session(&self, update: impl FnOnce(&mut ActiveLoginSession)) {
        if let Ok(mut sessions) = self.sessions.lock() {
            if let Some(session) = sessions.get_mut(&self.uuid) {
                if session.token == self.token {
                    update(session);
                }
            }
        }
    }
}

impl ActiveLoginRegistry {
    pub fn register_replacing(
        &self,
        uuid: &str,
        name: &str,
        stream: &TcpStream,
    ) -> io::Result<(ActiveLoginGuard, Option<TcpStream>)> {
        let token = self.next_token.fetch_add(1, Ordering::Relaxed);
        let stream = stream.try_clone()?;
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| io::Error::other("active login registry mutex poisoned"))?;
        let old = sessions
            .insert(
                uuid.to_string(),
                ActiveLoginSession {
                    token,
                    stream,
                    name: name.to_string(),
                    in_play: false,
                    allows_listing: false,
                    language: "en_us".to_string(),
                },
            )
            .map(|session| session.stream);

        Ok((
            ActiveLoginGuard {
                sessions: self.sessions.clone(),
                uuid: uuid.to_string(),
                token,
            },
            old,
        ))
    }

    /// Snapshot the players currently in the PLAY state for a status response, as
    /// `(uuid, name, allows_listing)` triples. Mirrors Java
    /// `MinecraftServer.buildPlayerStatus` reading `playerList.getPlayers()`:
    /// login/configuration-phase sessions are excluded.
    pub fn status_players(&self) -> Vec<StatusPlayer> {
        match self.sessions.lock() {
            Ok(sessions) => sessions
                .iter()
                .filter(|(_, session)| session.in_play)
                .map(|(uuid, session)| StatusPlayer {
                    uuid: uuid.clone(),
                    name: session.name.clone(),
                    allows_listing: session.allows_listing,
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Number of players currently in the PLAY state — the status online count
    /// (Java `playerList.getPlayers().size()`).
    pub fn online_count(&self) -> usize {
        match self.sessions.lock() {
            Ok(sessions) => sessions.values().filter(|session| session.in_play).count(),
            Err(_) => 0,
        }
    }

    /// Names of every player currently in the PLAY state, for the GS4 query
    /// response (Java `PlayerList.getPlayerNamesArray`). Unlike the status sample
    /// this is not filtered by listing preference — the query lists all players.
    pub fn online_player_names(&self) -> Vec<String> {
        match self.sessions.lock() {
            Ok(sessions) => sessions
                .values()
                .filter(|session| session.in_play)
                .map(|session| session.name.clone())
                .collect(),
            Err(_) => Vec::new(),
        }
    }
}
