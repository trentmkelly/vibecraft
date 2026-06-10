# CHECKLIST_LIGHTING.md

Full port of Minecraft 26.1.2's lighting engine to VibeCraft, replacing the current single-section `src/light.rs` stub with a multi-section, cross-chunk propagator that matches Java behavior 1:1.

This checklist is the authoritative scope for the lighting port. Every item must be completed. Do not mark an item complete unless the corresponding Rust code and tests exist and pass.

---

## Background

Today every chunk section is created with `sky_light = Some(vec![-1; 2048])` (all nibbles = 15) and `block_light = None`. The packet serializer (`src/network/play/chunk_c2.rs::ClientboundLightUpdatePacketData::from_chunk`) ships those arrays verbatim, which is why the world renders fullbright and the `[worldgen] chunk=… packet light=0ms` log shows zero work.

`src/light.rs` contains a single-section `LightEngine` that handles propagation only within one 16³ section. It is never invoked from the chunk pipeline. It must be replaced wholesale — not extended — because cross-section and cross-chunk propagation requires a fundamentally different data model (storage maps keyed by `SectionPos`, two-chunk neighbour cache, queued vs. live `DataLayer`s, retain semantics, section type lifecycle, etc.).

## Authoritative Java references

Consult these continuously. The decomp lives at `/home/trent/Projects/MinecraftServerCRB/decompiled-server-26.1.2`.

- `net/minecraft/world/level/lighting/LightEngine.java`
- `net/minecraft/world/level/lighting/BlockLightEngine.java`
- `net/minecraft/world/level/lighting/SkyLightEngine.java`
- `net/minecraft/world/level/lighting/LevelLightEngine.java`
- `net/minecraft/world/level/lighting/LayerLightSectionStorage.java`
- `net/minecraft/world/level/lighting/BlockLightSectionStorage.java`
- `net/minecraft/world/level/lighting/SkyLightSectionStorage.java`
- `net/minecraft/world/level/lighting/DataLayerStorageMap.java`
- `net/minecraft/world/level/lighting/ChunkSkyLightSources.java`
- `net/minecraft/world/level/lighting/LayerLightEventListener.java`
- `net/minecraft/world/level/lighting/LightEventListener.java`
- `net/minecraft/server/level/ThreadedLevelLightEngine.java` (`initializeLight`, `lightChunk`, `propagateLightSources` choreography)
- `net/minecraft/world/level/chunk/LightChunk.java` (`findBlockLightSources`)
- `net/minecraft/world/level/chunk/LightChunkGetter.java`
- `net/minecraft/world/level/chunk/DataLayer.java`
- `net/minecraft/world/level/chunk/status/ChunkStatusTasks.java` (INITIALIZE_LIGHT and LIGHT task wiring)
- `net/minecraft/world/level/LightLayer.java`
- `net/minecraft/core/SectionPos.java` (for the bit-packed long node format used everywhere in the engines)

---

## Section A — Core primitives

- [x] **A1. `DataLayer` parity primitives.** Provide a `DataLayer` value type that matches `net/minecraft/world/level/chunk/DataLayer.java`: 2048-byte nibble array, `y << 8 | z << 4 | x` index math, packed-filled lazy materialization, `set`/`get`/`copy`/`isEmpty`/`isDefinitelyHomogenous`/`isDefinitelyFilledWith`/`getData`. (The existing `DataLayer` in `src/light.rs` is a starting point but must move into the new module and gain `copy`, `isEmpty`, `isDefinitelyHomogenous`, and explicit "absent" representation.)
- [x] **A2. `SectionPos` packed-long encoding.** Provide helpers for `SectionPos::asLong`, `getZeroNode`, `offset`, `x/y/z` accessors, matching Java bit layout exactly. Used as the key throughout the light storage.
- [x] **A3. `LightLayer` enum** (`Block`, `Sky`) with `useful_light_for_player` matching Java semantics.
- [x] **A4. `Direction` light helpers.** Reuse existing direction utilities if present; otherwise add `PROPAGATION_DIRECTIONS`, opposites, and ordinal indices matching Java.
- [x] **A5. `QueueEntry` bitpacking** matching `LightEngine.QueueEntry`: 4 level bits, 6 direction bits, `FLAG_FROM_EMPTY_SHAPE`, `FLAG_INCREASE_FROM_EMISSION`, `decreaseAllDirections`, `decreaseSkipOneDirection`, `increaseLightFromEmission`, `increaseSkipOneDirection`, `increaseOnlyOneDirection`, `increaseSkySourceInDirections`, `withLevel`, `withDirection`, `withoutDirection`, `getFromLevel`, `isFromEmptyShape`, `isIncreaseFromEmission`, `shouldPropagateInDirection`.

## Section B — Storage layer

- [x] **B1. `DataLayerStorageMap` equivalent.** Long → `DataLayer` map with `copy()` semantics matching Java (the visible map vs. the working map swap pattern used by `LayerLightSectionStorage`). Track default per-layer absence.
- [x] **B2. `LayerLightSectionStorage` base behavior.** Track `dataSectionSet`, `toMarkNoData`, `toMarkData`, `queuedSections`, `untrustedSections`, `columnsWithSources`, `changedSections`, `hasInconsistencies`. Implement `getLightValue(long blockNode)`, `getStoredLevel`, `setStoredLevel`, `getDataLayer`, `getDataLayerToWrite`, `setLightEnabled`, `lightOnInColumn(long sectionZeroNode)`, `updateSectionStatus(long sectionNode, boolean isEmpty)`, `queueSectionData(long, @Nullable DataLayer)`, `retainData(long zeroNode, boolean retain)`, `swapSectionMap`, `markNewInconsistencies`. Expose `SectionType { EMPTY, LIGHT_AND_DATA, LIGHT_ONLY, TRUSTED }` with `display()` strings matching Java for debug output.
- [x] **B3. `BlockLightSectionStorage`** subclass tracking only block-light layers — `repeatFirstLayer` semantics, `createDataLayer`, `runAllUpdates` integration. Match the Java method overrides exactly.
- [x] **B4. `SkyLightSectionStorage`** subclass tracking the highest non-empty section per column and propagating sky sources downward through transparent empty sections (`getLightValue` walks up to the topmost stored layer when the column is light-on). Match `lightOnInColumn`, `enableLightSources`, and the inherited storage state machine.

## Section C — Light engines

- [x] **C1. Abstract `LightEngine` core.** Owns `chunkSource: LightChunkGetter`, `storage: S`, `blockNodesToCheck: LongOpenHashSet`, `increaseQueue`, `decreaseQueue`, two-chunk `lastChunkPos`/`lastChunk` cache. Implement `checkBlock`, `queueSectionData`, `retainData`, `updateSectionStatus`, `setLightEnabled`, `runLightUpdates` (drains blockNodesToCheck via `checkNode`, then `propagateDecreases`, `propagateIncreases`, clears cache, calls `storage.markNewInconsistencies(this)`, `storage.swapSectionMap`), `propagateIncreases`, `propagateDecreases`, `enqueueIncrease`, `enqueueDecrease`, `hasLightWork`, `getDataLayerData`, `getLightValue`, `getDebugData`, `getDebugSectionType`, abstract `checkNode`/`propagateIncrease`/`propagateDecrease`.
- [x] **C2. `getLightBlockInto`** with `useShapeForLightOcclusion` short-circuit, occlusion-shape merging via `Shapes.mergedFaceOccludes` parity (or a documented Rust shim that calls into the existing collision/voxel-shape infrastructure if present). If no voxel-shape support exists yet, gate behavior on the simple `lightDampening` value but make the API shape-aware so the shape path can be filled in later without a rewrite.
- [x] **C3. `hasDifferentLightProperties`** mirror of `LightEngine.hasDifferentLightProperties` (compares dampening, emission, occludesShape flags).
- [x] **C4. `BlockLightEngine`.** Concrete `checkNode`, `propagateIncrease`, `propagateDecrease`. Emission seeding via `LightChunk::find_block_light_sources` enumeration. Match the Java direction-skip logic precisely.
- [x] **C5. `SkyLightEngine`** with `ChunkSkyLightSources` per-column heightmap. Implement `propagateLightSources(ChunkPos)` that enumerates the column source y-values and enqueues the four cardinal + down increases via `increaseSkySourceInDirections`. Honor the "above the topmost section" fast path the same way Java does.
- [x] **C6. `ChunkSkyLightSources`.** Port `net/minecraft/world/level/lighting/ChunkSkyLightSources.java` — fixed-width packed array of lowest-sky-source Y per `(x, z)` column, `update(int x, int oldY, int newY)` semantics, `findLowestSourceY`. Driven from chunk block-state palettes; recomputed when blocks change state.
- [x] **C7. `LevelLightEngine`** top-level wrapper owning optional `blockEngine` + `skyEngine`. `checkBlock`, `hasLightWork`, `runLightUpdates`, `updateSectionStatus`, `setLightEnabled`, `propagateLightSources(ChunkPos)`, `queueSectionData`, `retainData`, `getRawBrightness`, `lightOnInColumn`, `getLightSectionCount`, `getMinLightSection`, `getMaxLightSection`. Exposes `LayerLightEventListener` accessors for each layer (with a dummy variant when the engine is absent — mirror Java's `DummyLightLayerEventListener.INSTANCE`).

## Section D — Chunk-pipeline glue

- [x] **D1. `LightChunk` trait** (Rust equivalent) implemented for `LevelChunk` — `getBlockState(BlockPos)`, `findBlockLightSources(BiConsumer<BlockPos, BlockState>)`. Block-state lookup must resolve via the existing paletted container code, not a hardcoded table.
- [x] **D2. `LightChunkGetter` trait** with `getChunkForLighting(int, int) -> Option<&dyn LightChunk>` and `getLevel()`. The chunk source used by the engines.
- [x] **D3. `LightBlock` via real block metadata.** Replace the current hardcoded `LightBlock::AIR` / `LightBlock::STONE` constants with a lookup that asks `block_metadata` (or block-state) for opacity (`lightDampening`), emission (`lightEmission`), and the `useShapeForLightOcclusion` flag for every block. Torches, glowstone, lava, sea lanterns, beacons, etc., must emit at their vanilla values. Leaves and glass must dampen by their vanilla values. Air must be transparent (opacity 0). Stone must occlude.
- [x] **D4. Generated-chunk lighting handoff.** After a chunk is fully generated (post terrain, carvers, surface, features), build a `LevelLightEngine` rooted at a `LightChunkGetter` that yields the freshly generated chunk (plus neighbours when available), call the equivalent of `ThreadedLevelLightEngine.initializeLight(chunk, false)` followed by `lightChunk(chunk, false)`, drain `runLightUpdates`, and write the resulting per-section block + sky `DataLayer`s back onto each `ChunkSection` via the existing `set_section_light_arrays`. Update `LevelChunk.light_correct = true` only when every non-empty section has both layers computed.
- [x] **D5. Wire into `write_generated_spawn_chunk_payload`.** When the chunk being serialized has `light_correct == false`, run the handoff (D4) before building the light packet data. When already correct (e.g. loaded from region with stored light arrays), skip recomputation.
- [x] **D6. Remove hardcoded `sky_light: Some(vec![-1; 2048])` seedings** in:
  - `src/worldgen/noise_chunk_fill.rs`
  - `src/worldgen/noise_preview_chunk.rs`
  - `src/worldgen/flat_generation.rs`
  - `src/storage/chunk.rs::set_block_state_raw` (the on-the-fly new-section path)
  
  Acceptance: `grep -rn "sky_light: Some(vec!\[-1" src/` returns zero matches.
- [x] **D7. Engine timing.** Expose a `light_update_count: u64` field on the engine result so the worldgen log line can prove propagation ran even when the wall-clock is sub-millisecond. Update the `[worldgen] chunk=…` log to include the count. (Sub-millisecond timing must not be hidden — the metric exists as a guard against future regressions where the engine silently no-ops.)

## Section E — Replace the stub

- [x] **E1. Delete `src/light.rs`** after moving any salvageable primitives (`DataLayer` bit math, `lowest_sky_source_y`, nibble packing) into the new module. No `mod light;` declaration remains.
- [x] **E2. Update every call site** of the old `light::` API to use the new module path. Run `grep -rn "use crate::light::\|crate::light::"` to confirm zero remaining references.
- [x] **E3. Delete or migrate the old tests** in `src/light.rs::tests` — equivalent behavior must be covered by the new test suite.
- [x] **E4. No feature flags, no parallel implementations, no "legacy mode".** One engine, one code path.

## Section F — Tests (sibling `tests.rs` per `project_vibecraft_binary_crate` memory)

Every test below must exist and pass. Tests live next to the module they exercise (e.g. `src/lighting/tests.rs` or per-file siblings).

- [x] **F1. DataLayer nibble parity** — round-trip set/get, packed-fill materialization, `isDefinitelyFilledWith`/`isEmpty`, byte index `index >> 1` and shift `4 * (index & 1)` match Java.
- [x] **F2. SectionPos packing** — known fixtures with Java-computed values.
- [x] **F3. QueueEntry round-trips** — every constructor/accessor pair, including `shouldPropagateInDirection` for every direction permutation.
- [x] **F4. Storage section-type lifecycle** — fresh section → EMPTY → LIGHT_AND_DATA on enable → LIGHT_ONLY after retain/unretain → back to EMPTY when disabled. `lightOnInColumn` flips correctly.
- [x] **F5. Block-light single-section** — torch at (8,8,8), no obstacles → 14,13,12,… decay. Existing `light::tests::block_light_propagates_from_emission_with_minimum_opacity` equivalent.
- [x] **F6. Block-light across section boundary** — torch at (8,15,8) lights (8,16,8) at level 13 (one block of air opacity + boundary cross).
- [x] **F7. Block-light across chunk boundary** — torch at (15, 64, 8) in chunk (0,0) bleeds into chunk (1,0) at (0,64,8) with the expected falloff.
- [x] **F8. Sky-light open column** — column with no occluders in a generated chunk → every voxel from `maxBuildHeight` down to `minBuildHeight` reports 15 sky-light.
- [x] **F9. Sky-light blocked column** — stone at y=k, air below → 15 above, 0 at y=k, 14 just below, 13 at y=k-2, decaying to 0.
- [x] **F10. Sky-light cardinal bleed** — open shaft next to overhang: voxels under the overhang receive 14/13/… from cardinal-bleed via `increaseSkySourceInDirections`.
- [x] **F11. Sky-light occlusion shape (slab/stairs).** When voxel-shape infrastructure is available, slabs in the upper half of a block let sky-light pass into the lower half. If voxel shapes are not yet wired, this test asserts the simple-dampening fallback documented in C2 — leaving a TODO comment is **not** acceptable; the test must reflect actual implemented behavior, and a follow-up checklist item must be added under Section H if shape support is deferred.
- [x] **F12. `LightEngine.runLightUpdates` accounting** — returns the expected node count; `hasLightWork` is false after drain; `clearChunkCache` runs.
- [x] **F13. `hasDifferentLightProperties`** — air↔stone, air↔glass, air↔torch, stone↔stone all match Java.
- [x] **F14. Block-state opacity sourcing (D3)** — assert that glass dampens by 0, leaves by 1, water by 1, stone by 15, lava by 0 (consult `data/minecraft/blocks` / block metadata for exact vanilla values).
- [x] **F15. Emission sourcing (D3)** — torch emits 14, glowstone 15, sea lantern 15, redstone torch 7, jack o'lantern 15. Values come from real block metadata.
- [x] **F16. End-to-end: generated chunk produces correct light packet.** Generate a flat-ish chunk with grass on top, stone underneath; run the full pipeline; deserialize the resulting `ClientboundLightUpdatePacketData`; assert:
  - Sky-light at `(0, surface+1, 0)` = 15.
  - Sky-light at `(0, surface-5, 0)` = 0 (or correctly decayed).
  - Block-light = 0 everywhere if no emissive blocks placed.
  - `empty_block_y_mask` is fully set (all sections present in empty mask, none in `block_updates`).
- [x] **F17. End-to-end with torch.** Place a glowstone block at `(8, surface+1, 8)`; assert the resulting packet has the corresponding section in `block_y_mask` and the data layer reads ≥14 at that position.
- [x] **F18. Regression — no fullbright leak.** Generate a chunk with a 16-block-deep ceiling at y=100; sky-light at y=80 in the covered area must be 0.
- [x] **F19. Generated chunk `light_correct` flag** — true after handoff completes for every section.

## Section G — Build, lint, integration verification

- [x] **G1. `cargo build` clean** from `VibeCraft/` (no warnings).
- [x] **G2. `cargo clippy --all-targets -- -D warnings` clean.**
- [x] **G3. `cargo test` passes** (no ignored or `#[cfg(off)]` lighting tests).
- [x] **G4. Worldgen log shows non-zero work.** Either `light=≥1ms` on a typical 16-section overworld chunk, or `light_update_count` field present in the log line with a plausible value (≥1024 nodes for a non-empty chunk). Whichever path is taken, the runtime guard against silent no-op is in place.
- [x] **G5. Verify in vanilla client.** Manual acceptance: caves are dark, surface is lit, torches glow. (Not automated, but explicitly noted as a sanity check before commit.)

## Section H — Documentation and checklist hygiene

- [x] **H1. Each new module has a doc comment** naming its Java counterpart (e.g. `//! Mirrors net.minecraft.world.level.lighting.BlockLightEngine.java`) and documenting invariants.
- [x] **H2. Update `CHECKLIST_WORLDGEN.md`**: tick items 257, 404, 775, 776, 777, 778, 779, 780, 781, each annotated with the Rust path that satisfies it. Do **not** touch any other checklist items (per `feedback_codex_checklists` memory — no batch-completing unrelated rows).
- [x] **H3. If C2 voxel-shape parity is deferred**, add a single new follow-up row to `CHECKLIST_WORLDGEN.md` (or to this file under a "Deferred" section) capturing exactly what is missing and why. Document it clearly — no silent gaps.
- [x] **H4. Single git commit on `new-features`** with a clear message ("Port Java 26.1.2 light engine: replace single-section stub with cross-chunk LevelLightEngine + Block/SkyLightEngine + storage + handoff"). User will push.

---

## Definition of done

All checkboxes above are ticked, all tests pass, build/clippy are clean, the worldgen log proves propagation ran, hardcoded `vec![-1; 2048]` seedings are gone, the single-section `src/light.rs` stub has been deleted, and a commit exists on `new-features`.
