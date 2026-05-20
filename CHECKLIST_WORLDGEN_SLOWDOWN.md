# Worldgen Slowdown Parity Checklist

Last updated: 2026-05-20

## Purpose

The real-surface worldgen path is currently usable only in the narrow sense that
it eventually produces terrain. Fresh chunks are still far too slow to generate:
recent live logs show roughly 10-13.5 seconds per fresh chunk, with the terrain
fill loop alone taking roughly 7.6-10.1 seconds and surface rules taking another
2.1-2.5 seconds.

This file is a methodical inventory of where RustCraft still diverges from the
Java server implementation. The goal is not to apply custom optimizations first.
The goal is to make the RustCraft generation shape match vanilla Java closely
enough that vanilla's known performance characteristics become plausible.

## Work Checklist

- [x] Add Java-equivalent `NoiseChunk` context counters:
  `interpolation_counter`, `array_interpolation_counter`, `array_index`, and
  `filling_cell`.
- [x] Replace the recursive static density evaluator with a first-class
  wrapped density-function runtime, or an equivalent layer with wrapper identity
  and per-wrapper state.
- [x] Implement wrapper-level scalar `compute(context)` and array
  `fill_array(output, context_provider)` dispatch.
  - [x] Add chunk-owned reusable scratch arrays for nested density
    `fill_array` calls, reducing temporary allocations while the full
    Java-style wrapper graph is still being built.
  - [x] Cache density-function value bounds per chunk so lazy scalar/array
    branches use Java-style precomputed min/max behavior instead of repeatedly
    walking static density trees.
  - [x] Match Java `RangeChoice.fillArray` branch shape: fill the selector
    array, then compute only the selected branch for each index instead of
    eagerly filling both branch arrays.
  - [x] Match Java two-argument `fillArray` branch shape for `mul`, `min`, and
    `max`: fill the first argument array, then evaluate the second argument
    lazily per index only when the Java short-circuit rules require it.
  - [x] Replace linear marker-wrapper lookup tables with Java-style identity
    maps so hot `Interpolated`, `CacheOnce`, `Cache2D`, `CacheAllInCell`, and
    `FlatCache` dispatch does not scan wrapper lists for every density sample.
- [x] Move `Interpolated` marker handling into the wrapped density-function
  layer.
  - [x] Match Java `NoiseInterpolator.compute` while `fillingCell` is true by
    returning direct `lerp3` cell values during cache fills instead of the last
    block-interpolation value.
  - [x] Match Java `NoiseInterpolator.fillArray`: when not filling a cell,
    delegate to the wrapped density function instead of sampling the current
    interpolator value.
- [x] Move `CacheAllInCell` marker handling into the wrapped density-function
  layer.
- [x] Implement Java-style `CacheOnce` scalar and array semantics.
  - [x] Replace exact-position-only scalar caching with Java-style
    `interpolation_counter` / `array_interpolation_counter` checks.
  - [x] Add an array-fill helper so top-level `CacheOnce` wrappers can copy a
    cached array for the same `array_interpolation_counter`.
  - [x] Copy cached `CacheOnce` arrays directly from wrapper state instead of
    cloning the cached array before copying to the output buffer.
  - [x] Add live `CacheOnce` scalar/array hit/miss counters to the worldgen log.
  - [x] Route nested `CacheOnce` wrappers through first-class wrapper
    `fill_array` dispatch instead of scalar recursion.
  - [x] Store `CacheOnce` as one wrapper-owned state object per collected
    marker input, matching Java's inner wrapper lifecycle instead of routing
    scalar and array cache state through a generic hash map.
- [x] Implement Java-style `FlatCache` precomputed quart-grid semantics.
- [x] Move `Cache2D` into the wrapped density-function layer.
  - [x] Store `Cache2D` as wrapper-owned `last_pos_2d` / `last_value` state,
    matching Java's inner `Cache2D` object instead of using a shared hash map.
- [x] Rework `fillSlice` to call wrapper `fill_array`, matching Java
  `NoiseInterpolator.fillArray`.
  - [x] Fill retained interpolator `slice0` / `slice1` rows in place instead
    of allocating a replacement 2D slice per interpolator pass, matching
    Java's retained `double[][]` slice storage.
  - [x] Cache terrain spline objects on `NoiseChunk` so spline density nodes
    behave more like Java's held object graph instead of rebuilding spline
    structures during recursive evaluation.
- [x] Rework `CacheAllInCell` fills to call wrapper `fill_array`, matching Java
  `cellCache.noiseFiller.fillArray(...)`.
- [x] Reuse wrapper-owned cell buffers for `CacheAllInCell`, full-noise, and
  vein-toggle fills instead of allocating fresh arrays for every selected cell.
- [x] Replace transitional linear marker lookup scans with Java-style identity
  maps while the full Java wrapper graph is still in progress. This keeps
  marker identity resolution explicit and avoids per-sample wrapper-list scans
  in the current transitional evaluator.
- [x] Rework full-noise cell filling to use the wrapped density-function layer.
  - [x] Match Java `ShiftedNoise.fillArray` by using direct context iteration
    instead of filling child shift arrays and then recomputing scalar values.
  - [x] Match Java `TransformerWithContext.fillArray` for
    `WeirdScaledSampler`: fill the input array once and transform each index
    with its context instead of recomputing the input scalar.
  - [x] Match Java `FindTopSurface.fillArray` by using direct context
    iteration instead of pre-filling child scratch arrays that Java never
    materializes for this node.
  - [x] Sample cached `NormalNoise` snapshots by reference in hot density
    paths, matching Java `RandomState` object reuse instead of cloning cached
    snapshot structs on every sample.
  - [x] Move wrapped density-runtime normal-noise sampling onto the active
    `NoiseChunk` cache so hot block and cell evaluations reuse noise snapshots
    directly instead of going through the thread-local fallback cache.
  - [x] Cache `BlendedNoise` snapshots on `NoiseChunk`, matching Java's
    object-graph reuse for the `old_blended_noise` density node instead of
    rebuilding the sampler from terrain random state at each wrapped sample.
  - [x] Add a Java-shaped single-point density evaluator for
    `preliminarySurfaceLevel`, so `Interpolated`, `CacheOnce`, and
    `CacheAllInCell` wrappers fall through to their wrapped functions when the
    context is not the active `NoiseChunk`.
- [x] Model the Java `MaterialRuleList` chain for aquifer, ore veins, and
  default block fallback.
- [x] Move aquifer density calls onto the wrapped router/context.
  - [x] Route per-block aquifer barrier pressure noise through the active
    `NoiseChunk` density context, matching Java's `barrierNoise.compute(context)`
    call in `Aquifer.NoiseBasedAquifer.calculatePressure`.
  - [x] Route cached aquifer fluid-status helper noise (`floodedness`, `spread`,
    `lava`, `erosion`, `depth`) through wrapped single-point contexts.
  - [x] Reuse `NoiseChunk.preliminarySurfaceLevel` from aquifer fluid-status
    computation instead of recomputing preliminary-surface noise directly.
- [x] Implement Java `NoiseChunk.maxPreliminarySurfaceLevel` behavior for
  aquifer `skipSamplingAboveY`.
- [x] Convert ore veins into a material-rule filler using wrapped vein density
  functions.
  - [x] Cache vein toggle/ridged/gap density arrays per selected cell instead
    of recursively sampling all three density functions for every solid block.
  - [x] Match Java `OreVeinifier` branch order by delaying positional-random
    creation until after Y range and veininess threshold checks pass.
  - [x] Cache only `veinToggle` per cell and defer `veinRidged`/`veinGap`
    evaluation until Java's later OreVeinifier branches can actually reach
    them.
  - [x] Move the remaining ore decision/random branch into a reusable
    Java-shaped material-rule filler.
  - [x] Remove the transitional per-cell `veinToggle` prefill and let
    `OreVeinifier` call `veinToggle.compute(context)` at block time like Java.
- [x] Add deeper timing/counter logs inside `fill_block_loop`.
  - [x] Break out final-density lookup, aquifer compute, ore density lookup,
    ore decision, aquifer call count, and ore sample count.
- [x] Rework surface generation around a reusable `SurfaceRules.Context`
  equivalent.
- [x] Make generated section block writes mutate paletted storage in place
  instead of unpacking and repacking the whole 4096-block section per block.
  Java's paletted container path is mutable; the old RustCraft path made every
  terrain and surface write scale with an entire section.
- [x] Reuse generated `BlockState`/NBT tags for terrain material writes instead
  of constructing a fresh block-state tag for every generated block.
- [x] Match Java's hot terrain write shape by resolving section/local-Y once per
  Y row and writing directly into that section, instead of recomputing
  world-coordinate section indices for every block write.
- [x] Wire real biome lookup into surface generation instead of hardcoded
  plains/temperature.
- [x] Cache surface biome lookup by quart coordinate inside the chunk, matching
  Java's quart-resolution biome sampling shape without repeatedly resampling
  identical biome positions during the surface pass.
- [x] Revisit chunk scheduling after fresh per-chunk generation cost became
  close enough to expose scheduling as the next bottleneck. RustCraft now
  keeps the configured view-distance radius but streams chunks as worker
  threads finish generation instead of preparing the whole square before
  sending the first chunk.
- [x] Add a direct Rust worldgen performance guard test that exercises the same
  real-surface spawn-chunk path as the server, so optimization passes no longer
  require manual client logins.
- [x] Enable light optimization for dev/test profiles. The same Rust worldgen
  path was ~2.7s in fully unoptimized test builds but ~180ms in release, so the
  remaining live-server slowdown was dominated by debug codegen rather than a
  Java worldgen algorithm divergence.
- [x] Match Java's full view-distance streaming behavior without reducing the
  server chunk radius. A temporary radius-2 workaround was rejected because it
  hid the slow generation path instead of moving us closer to vanilla parity.
  The join packet and chunk cache radius continue using `server.properties`
  view distance, while `src/network/status.rs` schedules generation across
  workers and writes chunks as they become ready. This follows the shape of
  Java's `NoiseBasedChunkGenerator.fillFromNoise`, which uses
  `CompletableFuture.supplyAsync(..., Util.backgroundExecutor().forName("wgen_fill_noise"))`.
- [x] Verify on a fresh world that spawn and nearby chunks generate without
  multi-second invisible/solid terrain.
  - [x] Add a direct Rust 3x3 spawn-area performance guard that generates the
    same real-surface `SPAWN` chunks used by the server and fails if total or
    per-chunk generation returns to multi-second timings.

## Current Baseline

Representative fresh-world timings after the first Java-structure pass:

- Spawn chunk `(0, 0)`: `terrain=10601ms`, `fill=8481ms`,
  `fill_noise_chunk_init=838ms`, `fill_block_loop=7629ms`,
  `fill_full_noise_cache=0ms`, `surface=2116ms`.
- Nearby chunks: `terrain=11373-13540ms`, `fill=9169-11183ms`,
  `fill_block_loop=8174-10157ms`, `surface=2149-2564ms`.
- `fill_full_noise_cache_fills=768` per chunk, but
  `fill_full_noise_cache` is only `0-154ms`, so the new full-noise cell cache is
  not the main problem.

RustCraft timing log location:

- `src/network/status.rs:4064-4095` prints the current detailed worldgen timing
  line.

## Java Source Reference Map

Use these decompiled Java sources as the primary reference:

- `../decompiled-server-26.1.2/net/minecraft/world/level/levelgen/NoiseBasedChunkGenerator.java`
  - `fillFromNoise`: lines 349-374.
  - `doFill`: lines 377-458.
- `../decompiled-server-26.1.2/net/minecraft/world/level/levelgen/NoiseChunk.java`
  - Constructor and wrapped router setup: lines 141-165.
  - `getInterpolatedState` / `getInterpolatedDensity`: lines 180-185.
  - `fillSlice`, `initializeForFirstCellX`, `advanceCellX`: lines 235-267.
  - `forIndex` / `fillAllDirectly`: lines 269-297.
  - `selectCellYZ`: lines 299-315.
  - `wrap` / `wrapNew`: lines 377-407.
  - `CacheAllInCell`: lines 529-570.
  - `CacheOnce`: lines 573-629.
  - `FlatCache`: lines 631-665.
- `../decompiled-server-26.1.2/net/minecraft/world/level/levelgen/Aquifer.java`
  - `NoiseBasedAquifer` constructor: lines 103-139.
  - `computeSubstance`: starts at lines 149-160.
  - Barrier pressure noise use: lines 306-360.
  - Fluid-level/deep-dark/lava noise use: lines 449-510.
- `../decompiled-server-26.1.2/net/minecraft/world/level/levelgen/SurfaceSystem.java`
  - `buildSurface`: lines 66-163.

Relevant RustCraft implementation points:

- `src/worldgen.rs:39381-39515`: recursive density evaluator
  `eval_density_fn_with_interp`.
- `src/worldgen.rs:39759-40140`: current `NoiseChunk` construction,
  interpolation, cell caches, and full-noise cache.
- `src/worldgen.rs:40200-40340`: current `fill_from_noise_chunk_inner_timed`
  fill loop and material selection.
- `src/worldgen.rs:27521-27690`: current `NoiseBasedAquifer` initialization and
  `compute_substance` start.
- `src/worldgen.rs:29551-29760`: current dynamic surface pass.

## Divergence Inventory

### 1. Density Function Execution Model

**Java behavior**

Java builds a wrapped runtime graph:

- `NoiseChunk` gets `NoiseRouter router = randomState.router()`.
- It creates `NoiseRouter wrappedRouter = router.mapAll(this::wrap)`.
- Every marker in the router is replaced by a stateful wrapper object via
  `wrapNew`.
- The wrapped objects are stored in a `wrapped` map keyed by density-function
  identity.

Reference:

- `NoiseChunk.java:141-143`
- `NoiseChunk.java:377-407`

**RustCraft behavior**

RustCraft currently does not build a wrapped runtime graph. Instead it:

- Walks static `DensityFunction` enums recursively in
  `eval_density_fn_with_interp`.
- Special-cases marker variants during that recursive walk.
- Uses pointer maps (`interp_by_ptr`, `cache_all_by_ptr`) to find selected
  tracked marker inputs.

Reference:

- `src/worldgen.rs:39381-39515`
- `src/worldgen.rs:39791-39858`

**Why this matters**

Java's wrappers carry state and expose `fillArray` behavior. RustCraft's
recursive evaluator behaves more like repeated point sampling. That creates a
large amount of repeated tree walking and misses Java's array-level cache
semantics.

**Status**

- Partially aligned.
- Still not equivalent.

**Next target**

Create a first-class runtime/wrapped density graph for a `NoiseChunk`, or an
equivalent dispatch layer where each wrapped function has:

- `compute(context)`
- `fill_array(output, context_provider)`
- marker-specific state
- Java-style identity deduplication

### 2. Full Noise Value / Material Rule Chain

**Java behavior**

Java creates:

```java
DensityFunction fullNoiseValue = DensityFunctions.cacheAllInCell(
    DensityFunctions.add(wrappedRouter.finalDensity(), DensityFunctions.BeardifierMarker.INSTANCE)
).mapAll(this::wrap);
this.fullNoiseDensity = fullNoiseValue;
builder.add(context -> this.aquifer.computeSubstance(context, fullNoiseValue.compute(context)));
builder.add(OreVeinifier.create(...));
this.blockStateRule = new MaterialRuleList(...);
```

References:

- `NoiseChunk.java:154-165`
- `NoiseChunk.java:180-185`
- `NoiseBasedChunkGenerator.java:427-435`

**RustCraft behavior**

RustCraft now has a per-cell `full_noise_values` cache, but the surrounding
material rule chain is still manually assembled in the fill loop:

- `noise_chunk.interpolated_density(...)`
- manual aquifer call
- manual ore vein call
- default block fallback

References:

- `src/worldgen.rs:39973-40049`
- `src/worldgen.rs:40112-40120`
- `src/worldgen.rs:40302-40340`

**Why this matters**

The full-noise cache itself is cheap in logs, but RustCraft still populates it
by recursively walking the static density tree. Java evaluates through wrapped
objects and uses the `MaterialRuleList` abstraction to keep the block-state
pipeline tied to that context.

**Status**

- Outer full-noise cache: partially implemented.
- Java `MaterialRuleList` shape: not implemented.

**Next target**

Model the Java block-state filler chain more directly:

- `fullNoiseDensity.compute(noise_chunk_context)`
- `aquifer.computeSubstance(context, density)`
- `OreVeinifier` as a material rule using wrapped vein functions
- default block fallback only after rule chain returns null

### 3. `fillSlice` / Interpolation Array Filling

**Java behavior**

Java fills interpolation slices by calling wrapper `fillArray` methods:

- `NoiseChunk.fillSlice` iterates z slices.
- It increments `arrayInterpolationCounter`.
- For each `NoiseInterpolator`, it calls `noiseInterpolator.fillArray(slice,
  sliceFillingContextProvider)`.

References:

- `NoiseChunk.java:235-252`
- `NoiseChunk.java:254-267`

**RustCraft behavior**

RustCraft recently changed the active slice fill path to route through
`eval_density_fn_with_interp`, but the older `NoiseInterpolatorState::fill_slice`
method still exists and performs raw `compute_with_noise` point sampling.

References:

- Old/raw method: `src/worldgen.rs:39610-39618`
- Active current path: `src/worldgen.rs:39935-39963`

**Why this matters**

Even the active RustCraft path still fills arrays by manually looping points and
calling the recursive evaluator. Java lets each wrapped density function decide
how to fill arrays. That is how `CacheOnce` can clone/copy entire previous
arrays and how wrappers can share work across repeated calls.

**Status**

- Raw point fill no longer appears to be the main active path after the latest
  change, but Java-style `fillArray` dispatch is still missing.

**Next target**

Replace manual point loops with a wrapper-level `fill_array` implementation.
Make array filling a behavior of density functions, not an external loop that
repeatedly calls scalar evaluation.

### 4. `CacheOnce` Semantics

**Java behavior**

Java `CacheOnce` has two layers:

- Scalar cache using `interpolationCounter`.
- Array cache using `arrayInterpolationCounter` and `arrayIndex`.

When the same array counter is active, Java can copy or index a cached array
instead of recomputing each point.

References:

- `NoiseChunk.java:573-629`
- `NoiseChunk.java:269-297`

**RustCraft behavior**

RustCraft currently stores only the last exact `(x, y, z)` scalar result per
cache key.

Reference:

- `src/worldgen.rs:39417-39431`
- `src/worldgen.rs:39749`

**Why this matters**

This is likely a major remaining terrain-fill slowdown. CacheOnce is used in
several density-function subgraphs, especially in cave/entrance/noodle-related
functions. Java avoids recomputing arrays during slice and cell fills; RustCraft
does not.

**Status**

- Implemented: RustCraft now scans the full aquifer grid every 4 blocks via
  `NoiseChunk::max_preliminary_surface_level`, matching Java's
  `NoiseChunk.maxPreliminarySurfaceLevel(...)` loop shape.

**Next target**

Add Java-equivalent state:

- `interpolation_counter`
- `array_interpolation_counter`
- `array_index`
- per-wrapper `last_counter`
- per-wrapper `last_value`
- per-wrapper `last_array_counter`
- per-wrapper `last_array`

Then make scalar `compute` and array `fill_array` honor these counters.

### 5. `FlatCache` Semantics

**Java behavior**

Java `FlatCache` precomputes a full chunk-local quart-coordinate grid:

- `sizeXZ = noiseSizeXZ + 1`.
- Values are filled for all quart X/Z positions in the chunk.
- Later compute calls index the prefilled array.

Reference:

- `NoiseChunk.java:631-665`

**RustCraft behavior**

RustCraft currently treats `FlatCache` as a single-entry cache keyed by
quantized X/Z.

Reference:

- `src/worldgen.rs:39433-39449`
- `src/worldgen.rs:39750`

**Why this matters**

Flat 2D climate/spline helper functions are sampled repeatedly. Java pays the
chunk-local setup cost once and then does array indexing. RustCraft can evict
or miss constantly as columns change.

**Status**

- Divergent.

**Next target**

Create per-wrapper `FlatCache` state with a precomputed `(noise_size_xz + 1)^2`
array. Use Java's quart conversion/indexing rules.

### 6. `Cache2D` Semantics

**Java behavior**

Java wraps `Cache2D` as a stateful density function object. Its exact internals
are part of the same wrapper/counter model and are invoked through
`compute`/`fillArray`.

Reference:

- `NoiseChunk.java:382-389`

**RustCraft behavior**

RustCraft currently uses a generic `cache_2d_values` map, keyed by function
identity and `pack_column(x, z)`.

Reference:

- `src/worldgen.rs:39402-39415`
- `src/worldgen.rs:39750`

**Why this matters**

The cache is global-ish within the `NoiseChunk`, not attached to a wrapper
object with Java's lifecycle. It may be functionally close for some scalar
cases, but it is not equivalent during array filling.

**Status**

- Probably partially correct for scalar calls.
- Divergent for array fills and wrapper identity/lifecycle.

**Next target**

Move `Cache2D` into the wrapped density-function layer.

### 7. `CacheAllInCell` Semantics

**Java behavior**

Java `CacheAllInCell` registers itself in `NoiseChunk.cellCaches`. On
`selectCellYZ`, Java calls:

```java
cellCache.noiseFiller.fillArray(cellCache.values, this);
```

The cache then serves values by `arrayIndex`/in-cell coordinates.

References:

- `NoiseChunk.java:299-315`
- `NoiseChunk.java:529-570`

**RustCraft behavior**

RustCraft collects `CacheAllInCell` marker inputs, then fills values with a
manual nested loop and scalar recursive evaluation.

References:

- `src/worldgen.rs:39732-39734`
- `src/worldgen.rs:39994-40022`

**Why this matters**

The number of fills matches Java's cell structure (`768` per chunk in current
logs), but the fill method is not Java-equivalent. Any inner `CacheOnce` or
other wrapper that should have array semantics does not get that behavior.

**Status**

- Cell count and storage shape partially aligned.
- Fill semantics divergent.

**Next target**

Make `CacheAllInCell` use wrapped `fill_array` rather than scalar recursive
evaluation.

### 8. Interpolation Counters And Context State

**Java behavior**

Java maintains:

- `interpolationCounter`
- `arrayInterpolationCounter`
- `arrayIndex`
- `fillingCell`
- `inCellX/Y/Z`
- `cellStartBlockX/Y/Z`

These counters are central to `CacheOnce`, `CacheAllInCell`, interpolation, and
array filling.

References:

- `NoiseChunk.java:235-315`
- `NoiseChunk.java:573-629`

**RustCraft behavior**

RustCraft has position state and `filling_cell_cache`, but does not yet carry
the full Java counter model.

References:

- `src/worldgen.rs:39736-39752`
- `src/worldgen.rs:39935-40049`

**Why this matters**

Without equivalent counters, cache wrappers cannot know when a scalar value or
array value is still valid in the Java sense.

**Status**

- Divergent.

**Next target**

Add the Java counters before finishing wrapper cache parity.

### 9. Aquifer Uses Raw Router Functions

**Java behavior**

Java constructs `Aquifer.NoiseBasedAquifer` with the wrapped router:

- `Aquifer.create(this, ..., wrappedRouter, ...)`.
- Aquifer stores wrapped barrier, floodedness, spread, lava, erosion, and depth
  functions.

References:

- `NoiseChunk.java:141-151`
- `Aquifer.java:103-119`
- `Aquifer.java:306-360`
- `Aquifer.java:449-510`

**RustCraft behavior**

RustCraft creates `NoiseBasedAquifer` outside `NoiseChunk`, stores raw
`NoiseRouter`, and uses raw `compute_with_noise` paths inside aquifer helpers.

References:

- `src/worldgen.rs:27521-27605`
- `src/worldgen.rs:40253-40268`

**Why this matters**

Aquifer currently bypasses the wrapper/cache model. Even if direct aquifer
initialization timing is low, aquifer work is embedded in the block loop. Any
barrier/floodedness/deep-dark/lava checks can repeatedly compute raw density
functions instead of using Java's wrapped context.

**Status**

- Divergent.

**Next target**

Move aquifer ownership into `NoiseChunk` or pass it a wrapped-router/context
facade so all aquifer density calls use the same wrapper layer.

### 10. Aquifer `skipSamplingAboveY`

**Java behavior**

Java computes `skipSamplingAboveY` from:

```java
noiseChunk.maxPreliminarySurfaceLevel(fromGridX(minGridX, 0),
                                      fromGridZ(minGridZ, 0),
                                      fromGridX(maxGridX, 9),
                                      fromGridZ(maxGridZ, 9))
```

References:

- `Aquifer.java:134-138`

**RustCraft behavior**

RustCraft samples four corners only.

Reference:

- `src/worldgen.rs:27553-27589`

**Why this matters**

This is both a parity issue and a possible performance issue. If the Rust value
is too low, aquifer logic may run in positions where Java would use the global
fluid fast path.

**Status**

- Divergent.

**Next target**

Implement `NoiseChunk::maxPreliminarySurfaceLevel` with Java's sampled range
behavior and use it in aquifer initialization.

### 11. Ore Vein Rule Shape

**Java behavior**

Java adds ore veins as a `BlockStateFiller`:

```java
builder.add(OreVeinifier.create(wrappedRouter.veinToggle(),
                                wrappedRouter.veinRidged(),
                                wrappedRouter.veinGap(),
                                randomState.oreRandom()));
```

Reference:

- `NoiseChunk.java:161-163`

**RustCraft behavior**

RustCraft evaluates vein functions manually inside the fill loop after aquifer
logic returns a solid slot.

Reference:

- `src/worldgen.rs:40331-40340` and following ore-decision code.

**Why this matters**

Manual evaluation is close in high-level behavior, but it bypasses Java's
material-rule object shape. It also depends on the recursive evaluator and the
partial marker cache system.

**Status**

- Partially aligned behaviorally.
- Divergent structurally.

**Next target**

Represent `OreVeinifier` as a material rule using wrapped vein density
functions.

### 12. Surface Rule Execution Model

**Java behavior**

Java builds one reusable context and one applied rule per chunk:

- `SurfaceRules.Context context = new SurfaceRules.Context(...)`.
- `SurfaceRules.SurfaceRule rule = ruleSource.apply(context)`.
- Per column: `context.updateXZ(...)`.
- Per Y: `context.updateY(...)`, then `rule.tryApply(...)`.

References:

- `SurfaceSystem.java:66-163`

**RustCraft behavior**

RustCraft uses a dynamic rule tree evaluator:

- Preloads surface noise.
- Constructs `BuildSurfaceColumnState`.
- Recursively calls `dyn_surface_rule_apply` per block sample.
- Hardcodes biome as plains and temperature as `0.8`.

References:

- `src/worldgen.rs:29551-29760`

**Why this matters**

Surface is consistently about 2.1-2.5 seconds per chunk in current logs. That
is secondary to fill-loop cost but still too slow. The recursive dynamic
evaluator likely repeats work that Java stores in context/rule objects.

**Status**

- Functionally partial.
- Performance and biome parity divergent.

**Next target**

Build a reusable Rust `SurfaceRules.Context` equivalent and pre-apply the rule
source once per chunk. Add cached condition/rule state matching Java's surface
rule classes.

### 13. Biome Lookup During Surface

**Java behavior**

Java looks up the real surface biome:

```java
Holder<Biome> surfaceBiome = biomeManager.getBiome(...)
```

It then handles badlands and frozen ocean special cases.

References:

- `SurfaceSystem.java:112-159`

**RustCraft behavior**

RustCraft hardcodes:

- `biome = "minecraft:plains"`
- `temperature = 0.8`

Reference:

- `src/worldgen.rs:29683-29684`

**Why this matters**

This is not likely the main slowdown, but it is a clear parity gap and can
change surface materials substantially.

**Status**

- Divergent.

**Next target**

Wire real biome source sampling into surface generation before treating surface
parity as complete.

### 14. Chunk Generation Scheduling

**Java behavior**

Java schedules fill work asynchronously:

```java
CompletableFuture.supplyAsync(..., Util.backgroundExecutor().forName("wgen_fill_noise"))
```

Reference:

- `NoiseBasedChunkGenerator.java:349-374`

**RustCraft behavior**

RustCraft has a generated chunk cache and pre-generation batching in the network
path, but chunk generation still appears effectively limited by expensive
per-chunk synchronous work. The cache helps reuse and some parallel preparation,
but it cannot hide a 10+ second per-chunk generator.

References:

- `src/network/status.rs` generated chunk cache and `prepare_many`.
- `src/network/status.rs:4064-4095` worldgen timing output.

**Why this matters**

Scheduling matters for player experience, but current per-chunk cost is too
high for scheduling alone to solve. Do not treat async generation as the primary
fix until fill/surface parity is closer.

**Status**

- Partially implemented.
- Not the root cause of current slowdown.

**Next target**

Return to scheduling after per-chunk fresh generation is reasonably close to
vanilla.

### 15. Instrumentation Gaps

Current timing shows major phase totals, but not enough detail inside the fill
block loop.

Known current fields:

- `fill_noise_chunk_init`
- `fill_aquifer_init`
- `fill_block_loop`
- `fill_full_noise_cache`
- `surface_column_loop`
- mob and heightmap breakdowns

Reference:

- `src/network/status.rs:4064-4095`

Missing breakdowns needed before the next optimization pass:

- Interpolator slice fill time, separate from `fill_noise_chunk_init` and
  `advance_cell_x`.
- `CacheAllInCell` fill time by marker/function.
- `CacheOnce` hits/misses, once implemented.
- `FlatCache` construction time and hit count, once implemented.
- Aquifer `compute_substance` total time inside `fill_block_loop`.
- Aquifer sub-breakdown:
  - global fast path
  - skip-sampling fast path
  - aquifer cell location lookup
  - `computeFluid`
  - barrier pressure
  - random/source noise calls
- Ore vein total time and call count.
- Dynamic surface rule apply time versus block get/set time.

## Recommended Order Of Work

1. Add Java counters to `NoiseChunk`:
   - `interpolation_counter`
   - `array_interpolation_counter`
   - `array_index`
   - `filling_cell`

2. Implement a wrapped density-function runtime layer:
   - wrapper identity map
   - scalar `compute`
   - array `fill_array`
   - marker wrappers for `Interpolated`, `CacheAllInCell`, `CacheOnce`,
     `Cache2D`, and `FlatCache`

3. Move current scalar cache logic out of generic hash maps and into wrappers.

4. Rework `fillSlice`, `CacheAllInCell`, and full-noise cache filling to call
   wrapper `fill_array`, matching Java.

5. Move aquifer density calls onto the wrapped router/context.

6. Implement Java `maxPreliminarySurfaceLevel` behavior for aquifer
   `skipSamplingAboveY`.

7. Convert ore veins into a material-rule filler using wrapped vein functions.

8. Rework surface generation around a reusable `SurfaceRules.Context` and
   applied rule object.

9. Only after per-chunk fresh generation is much closer to vanilla, revisit
   network scheduling and chunk send strategy.

## Acceptance Criteria

The slowdown work should not be considered done until all of the following are
true on a fresh world:

- Spawn chunk generation no longer takes multiple seconds.
- Nearby fresh chunks do not remain invisible/solid for noticeable periods.
- `fill_block_loop` is no longer the dominant multi-second phase.
- `surface_column_loop` is no longer a multi-second phase.
- Logs include enough detail to identify any remaining hotspot without adding a
  new timer pass.
- The implementation shape is recognizably close to the Java source cited
  above, especially for `NoiseChunk` wrapper/counter behavior.
