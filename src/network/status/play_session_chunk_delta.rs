use super::*;

pub struct PlayChunkDeltaRequest<'a> {
    pub center: ChunkPos,
    pub chunks: &'a [(i32, i32)],
    pub update_cache_center: bool,
    pub world_root: &'a Path,
    pub world_seed: i64,
    pub chunk_cache: &'a GeneratedChunkCache,
    pub live_fluid_ticks: Option<(&'a mut LiveFluidTicks, i64, &'a WorldLayout)>,
}

pub fn write_play_chunk_delta(
    stream: &mut TcpStream,
    compression: CompressionState,
    mut request: PlayChunkDeltaRequest<'_>,
) -> io::Result<()> {
    let batch_started = Instant::now();
    log_chunk_batch_start(&request);
    write_chunk_cache_center_if_needed(
        stream,
        compression,
        request.update_cache_center,
        request.center,
    )?;
    if request.chunks.is_empty() {
        log_chunk_batch_finish(request.center, 0, batch_started);
        return Ok(());
    }
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_PLAY_CHUNK_BATCH_START_PACKET_ID,
        |_payload| Ok(()),
    )?;
    write_play_chunk_delta_chunks(stream, compression, &mut request, batch_started)?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_PLAY_CHUNK_BATCH_FINISHED_PACKET_ID,
        |payload| write_var_i32(payload, request.chunks.len() as i32),
    )?;
    log_chunk_batch_finish(request.center, request.chunks.len(), batch_started);
    Ok(())
}

fn log_chunk_batch_start(request: &PlayChunkDeltaRequest<'_>) {
    eprintln!(
        "[chunk-batch-timing] start center=({}, {}) chunks={} update_center={} live_fluid_seed={}",
        request.center.x,
        request.center.z,
        request.chunks.len(),
        request.update_cache_center,
        request.live_fluid_ticks.is_some()
    );
}

fn write_chunk_cache_center_if_needed(
    stream: &mut TcpStream,
    compression: CompressionState,
    update_cache_center: bool,
    center: ChunkPos,
) -> io::Result<()> {
    if !update_cache_center {
        return Ok(());
    }
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID,
        |payload| {
            write_var_i32(payload, center.x)?;
            write_var_i32(payload, center.z)
        },
    )
}

fn write_play_chunk_delta_chunks(
    stream: &mut TcpStream,
    compression: CompressionState,
    request: &mut PlayChunkDeltaRequest<'_>,
    batch_started: Instant,
) -> io::Result<()> {
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
    let worker_count = request.chunks.len().min(workers);
    let queue = Arc::new(Mutex::new(VecDeque::from(request.chunks.to_vec())));
    let (sender, receiver) = mpsc::channel::<(i32, i32, Arc<LevelChunk>)>();
    thread::scope(|scope| {
        for _ in 0..worker_count {
            let queue = Arc::clone(&queue);
            let sender = sender.clone();
            let cache = request.chunk_cache.clone();
            let world_root = request.world_root;
            let world_seed = request.world_seed;
            scope.spawn(move || loop {
                let next = match queue.lock() {
                    Ok(mut queue) => queue.pop_front(),
                    Err(_) => None,
                };
                let Some((x, z)) = next else {
                    break;
                };
                let chunk = cache.get_or_load(x, z, world_root, world_seed);
                if sender.send((x, z, chunk)).is_err() {
                    break;
                }
            });
        }
        drop(sender);

        for received in 0..request.chunks.len() {
            let recv_started = Instant::now();
            let (_x, _z, chunk) = receiver.recv().map_err(|err| {
                io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    format!("chunk generation worker stopped before batch completed: {err}"),
                )
            })?;
            let recv_ms = recv_started.elapsed().as_millis();
            let write_started = Instant::now();
            if let Some((ticks, game_time, _layout)) = request.live_fluid_ticks.as_mut() {
                unpack_chunk_fluid_ticks(ticks, *game_time, &chunk);
            }
            write_generated_spawn_chunk_packets_from_chunk(stream, compression, &chunk)?;
            let write_ms = write_started.elapsed().as_millis();
            if write_ms >= 10 || recv_ms >= 10 || received + 1 == request.chunks.len() {
                eprintln!(
                    "[chunk-batch-timing] progress center=({}, {}) sent={}/{} chunk=({}, {}) recv_wait={}ms write={}ms elapsed={}ms",
                    request.center.x,
                    request.center.z,
                    received + 1,
                    request.chunks.len(),
                    chunk.pos.x,
                    chunk.pos.z,
                    recv_ms,
                    write_ms,
                    batch_started.elapsed().as_millis()
                );
            }
        }
        Ok::<(), io::Error>(())
    })
}

fn log_chunk_batch_finish(center: ChunkPos, chunk_count: usize, batch_started: Instant) {
    eprintln!(
        "[chunk-batch-timing] finish center=({}, {}) chunks={} elapsed={}ms",
        center.x,
        center.z,
        chunk_count,
        batch_started.elapsed().as_millis()
    );
}
