# Chunk Generation and Sending Architecture Checklist

Goal: replace RustCraft's blocking full-view-distance chunk batch with a Java-parity asynchronous chunk generation and paced chunk sending pipeline, without reducing view distance or papering over timeouts.

Authoritative Java references:

- `decompiled-server-26.1.2/net/minecraft/server/level/ChunkMap.java`
- `decompiled-server-26.1.2/net/minecraft/server/level/GenerationChunkHolder.java`
- `decompiled-server-26.1.2/net/minecraft/server/level/ChunkGenerationTask.java`
- `decompiled-server-26.1.2/net/minecraft/server/level/ChunkTaskDispatcher.java`
- `decompiled-server-26.1.2/net/minecraft/server/level/ChunkTaskPriorityQueue.java`
- `decompiled-server-26.1.2/net/minecraft/server/network/PlayerChunkSender.java`
- `decompiled-server-26.1.2/net/minecraft/server/MinecraftServer.java`
- `decompiled-server-26.1.2/net/minecraft/world/level/chunk/status/ChunkStatusTasks.java`

## Baseline and diagnostics

- [ ] Preserve current `[chunk-batch-timing]` and `[fluid-timing]` diagnostics until the new pipeline is validated.
- [ ] Add a focused diagnostic summarizing per-player pending chunks, ready chunks, generated chunks, and sent chunks per tick.
- [ ] Add a diagnostic for chunk generation worker queue depth and oldest queued request age.
- [ ] Add a diagnostic for chunk send pacing: batch size, unacknowledged batches, desired chunks per tick, and batch quota.
- [ ] Document the current failure mode in code comments: blocking initial batch of 440 chunks at view distance 10 causes login timeout because the play loop cannot tick while waiting.

## Java behavior model

- [ ] Port the conceptual split from Java: chunk tracking determines what should be visible; generation prepares chunks asynchronously; sending drains ready chunks in paced batches.
- [ ] Mirror Java `PlayerChunkSender` semantics: pending chunks are tracked per player/connection and are not sent all at once.
- [ ] Mirror Java's nearest-first send ordering using distance squared from the player's current chunk.
- [ ] Mirror Java's initial `desiredChunksPerTick = 9.0` pacing model.
- [ ] Mirror Java's client feedback hook from `ServerboundChunkBatchReceivedPacket` to update desired chunks per tick.
- [ ] Mirror Java's unacknowledged batch limit behavior instead of blocking until all requested chunks are sent.
- [ ] Mirror Java's `ClientboundChunkBatchStartPacket` and `ClientboundChunkBatchFinishedPacket` around each sent batch, not around the entire view-distance set.
- [ ] Preserve `ClientboundSetChunkCacheCenterPacket` behavior when the player changes chunk center.
- [ ] Preserve `ClientboundForgetLevelChunkPacket` behavior for chunks leaving the tracking view.

## Data structures

- [ ] Introduce a per-play-session pending chunk sender state analogous to Java `PlayerChunkSender`.
- [ ] Store pending chunks as packed chunk positions or `ChunkPos` values with O(1) membership/removal.
- [ ] Track `desired_chunks_per_tick`, `batch_quota`, `unacknowledged_batches`, and `max_unacknowledged_batches` per session.
- [ ] Track sent-but-not-acked batch count separately from chunk generation readiness.
- [ ] Track visible/loaded chunk window separately from pending-to-send chunks.
- [ ] Keep `GeneratedChunkCache` as a cache of completed chunks, not as a synchronous loading API for the send path.
- [ ] Add a shared async chunk generation coordinator for all sessions so duplicate requests for the same chunk coalesce.
- [ ] Store chunk generation state as `Pending`, `Ready(Arc<LevelChunk>)`, or `Failed`.
- [ ] Add cancellation or deprioritization hooks for chunks that leave every player's visible range before generation completes.
- [ ] Ensure chunk generation state does not hold `TcpStream` or per-player mutable state.

## Async generation scheduler

- [ ] Replace blocking `cache.get_or_load` use in chunk send paths with nonblocking readiness checks.
- [ ] Add `request_chunk(pos, world_root, world_seed)` that schedules generation if not cached or already pending.
- [ ] Add `try_get_ready_chunk(pos)` that returns immediately with `None` when generation is not complete.
- [ ] Use a bounded worker pool for real chunk generation.
- [ ] Coalesce duplicate chunk requests from multiple players into one generation job.
- [ ] Prioritize chunks by ticket/view distance from active players, matching Java's ticket-level priority concept where practical.
- [ ] Ensure generation workers never write directly to a client socket.
- [ ] Ensure generation workers can send completion notifications through a channel to the play loop or cache manager.
- [ ] Ensure generation failures produce an empty/fallback chunk only through an explicit error path with logging.
- [ ] Preserve existing `load_or_generate_spawn_chunk_uncached` behavior for direct diagnostic/test paths unless deliberately migrated.

## Play loop integration

- [ ] Keep the play loop responsive while chunks are generating.
- [ ] Process keepalives, inbound movement, inventory, food ticks, water ticks, weather/time sync, and item pickups while chunks are pending.
- [ ] Move chunk sending into a per-tick drain step similar to Java `MinecraftServer` calling `chunkSender.sendNextChunks(player)`.
- [ ] On login, enqueue the visible chunk window instead of generating and sending it synchronously.
- [ ] Send the center chunk with priority but do not block indefinitely waiting for the whole radius.
- [ ] On player chunk movement, diff old/new visible windows and enqueue newly visible chunks.
- [ ] On player chunk movement, send forget packets for chunks that leave the visible window and remove them from pending send state.
- [ ] Avoid holding locks while writing packets to the network.
- [ ] Avoid holding locks while running worldgen.
- [ ] Ensure chunk cache center and radius packets still precede chunk payload packets as required by the client.

## Packet pacing and client acknowledgement

- [ ] Implement `ServerboundChunkBatchReceivedPacket` handling to update per-session desired chunks per tick.
- [ ] Clamp desired chunks per tick to Java's `0.01..=64.0` range.
- [ ] Start sessions at Java's default `9.0` chunks per tick.
- [ ] Initialize or reset batch quota to Java's behavior when all outstanding batches are acknowledged.
- [ ] Do not send new chunk batches when unacknowledged batches exceed the current cap.
- [ ] Start with Java's conservative max unacknowledged batch behavior and update it after the first client acknowledgement.
- [ ] Emit `ClientboundChunkBatchStartPacket` only when at least one ready chunk will be sent.
- [ ] Emit `ClientboundChunkBatchFinishedPacket(count)` with the number of chunks actually sent in that batch.
- [ ] Remove chunks from pending only after they are included in a sent batch.
- [ ] Keep chunks pending if generation is not ready yet.

## Chunk selection and ordering

- [ ] Select ready chunks nearest to the player's current chunk first.
- [ ] Keep not-yet-ready chunks pending for later ticks instead of blocking the batch.
- [ ] Limit each batch by current batch quota.
- [ ] If pending count is less than quota, send all ready pending chunks sorted nearest-first.
- [ ] If pending count exceeds quota, choose the nearest ready chunks up to quota.
- [ ] Re-evaluate player position each tick before selecting chunks, because the player may move while generation is pending.
- [ ] Avoid sending chunks outside the current tracking view even if they finished generation late.
- [ ] Avoid sending duplicate chunk payloads for the same pending chunk.

## Cache and region storage

- [ ] Reuse generated chunks across sessions through the shared cache.
- [ ] Ensure region-loaded chunks and generated chunks share the same ready-cache pathway.
- [ ] Keep region read failures isolated to the affected chunk.
- [ ] Do not synchronously save generated chunks on the network send path unless explicitly required.
- [ ] Preserve current chunk invalidation behavior for block/fluid mutations.
- [ ] Ensure invalidated chunks can be regenerated or reloaded without stale ready-cache entries.
- [ ] Confirm live fluid block updates invalidate the shared ready cache consistently.

## Fluid tick interaction

- [ ] Keep live fluid scheduling independent of chunk send completion.
- [ ] Seed live fluid ticks when a chunk becomes ready or is sent, whichever is the correct server-side parity point after Java review.
- [ ] Avoid scanning all 440 chunks for fluid edges synchronously on login.
- [ ] If fluid seeding stays tied to send, perform it incrementally per sent batch.
- [ ] Keep `[fluid-timing]` diagnostics around seeding and ticking until the final behavior is stable.
- [ ] Confirm generated natural water/lava still begins flowing after the chunk becomes active/ticking.

## Thread safety

- [ ] Audit all data shared between worldgen workers and the play loop.
- [ ] Store completed chunks in `Arc<LevelChunk>` or equivalent immutable shared structures.
- [ ] Do not mutate `LevelChunk` from worker threads after it is published as ready.
- [ ] Protect shared maps with small critical sections only.
- [ ] Avoid nested locks involving chunk cache, pending generation map, and network/session state.
- [ ] Ensure worker panic/error cannot poison the whole server session.
- [ ] Ensure disconnect cleans up per-session pending chunk state without cancelling globally useful generation jobs incorrectly.

## Tests

- [ ] Add a unit test for chunk sender quota behavior using Java default `desiredChunksPerTick = 9.0`.
- [ ] Add a unit test that pending chunks are selected nearest-first.
- [ ] Add a unit test that not-ready chunks stay pending and do not block ready chunks.
- [ ] Add a unit test for client batch acknowledgement updating desired chunks per tick with clamping.
- [ ] Add a unit test for unacknowledged batch gating.
- [ ] Add a unit test that moving the player drops stale pending chunks outside the new tracking view.
- [ ] Add a unit test that duplicate chunk requests coalesce into a single generation job.
- [ ] Add an integration-style test or deterministic harness showing login scheduling does not synchronously generate the full view-distance square.
- [ ] Add a regression test that initial join can produce gameplay ticks before the full visible radius is generated.

## Migration steps

- [ ] Extract current blocking chunk batch code behind a small interface so behavior can be swapped safely.
- [ ] Introduce the per-session chunk sender state without changing behavior yet.
- [ ] Add nonblocking ready-cache APIs beside existing blocking cache APIs.
- [ ] Add async generation scheduling and completion cache.
- [ ] Change login to enqueue chunks instead of synchronously sending the full window.
- [ ] Change movement chunk-delta handling to enqueue/drops chunks instead of blocking on newly visible chunks.
- [ ] Add per-tick chunk batch draining.
- [ ] Wire client chunk batch acknowledgements into sender pacing.
- [ ] Remove or restrict the old blocking batch path from live play code after parity behavior is verified.
- [ ] Keep direct blocking generation available only for tests/tools that explicitly need it.

## Validation plan

- [ ] Build with `RUSTCRAFT_SKIP_LINE_CHECK=1` after each major phase.
- [ ] Run targeted chunk sender unit tests after implementing sender state.
- [ ] Run targeted generation scheduler tests after implementing async generation.
- [ ] Preserve current `normal_overworld_generation_keeps_vanilla_block_array_parity_above_threshold` score or improve it; current baseline is `0.995887 (293699/294912)`, and no implementation step may lower this KPI.
- [ ] Preserve current `normal_overworld_generation_keeps_vanilla_column_profile_parity_above_threshold` score or improve it; current baseline is `0.423177 (325/768)`, and no implementation step may lower this KPI.
- [ ] Preserve current `normal_overworld_generation_keeps_vanilla_heightmap_parity_above_threshold` score or improve it; current baseline is `0.593750 (456/768)`, and no implementation step may lower this KPI.
- [ ] Preserve current `real_surface_spawn_chunk_generation_stays_under_debug_budget` timing or improve it; current baseline is `148ms` for spawn chunk `(0,0)` against the `4ms` debug budget, and no implementation step may make this KPI slower.
- [ ] Run the three normal-overworld vanilla parity KPI tests after each chunking architecture phase that can affect generated chunks, chunk caching, or chunk send readiness.
- [ ] Run `real_surface_spawn_chunk_generation_stays_under_debug_budget` after each chunking architecture phase that can affect chunk generation scheduling, caching, or worldgen execution time.
- [ ] Start a fresh world with default view distance and confirm join no longer blocks for the full 440 chunk batch.
- [ ] Confirm server continues sending keepalives and processing movement while chunks are pending.
- [ ] Confirm chunk packets arrive progressively and terrain fills in around the player.
- [ ] Confirm no view-distance cap or radius reduction was introduced.
- [ ] Confirm fluid flow still works after chunk pipeline changes.
- [ ] Remove or downgrade temporary high-volume timing logs after the architecture is stable.
