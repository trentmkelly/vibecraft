# Vanilla Worldgen Parity Checklist

## Relevant Java Source Files

- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/NoiseBasedChunkGenerator.java` — main noise-based generator
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/FlatLevelSource.java` — flat world generator
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/DebugLevelSource.java` — debug world generator
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/NoiseChunk.java` — per-chunk noise sampling
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/NoiseGeneratorSettings.java` — noise settings
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/NoiseRouter.java` — density function router
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/NoiseRouterData.java` — noise router bootstrap
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/DensityFunctions.java` — all density function types
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/DensityFunction.java` — density function interface
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/Aquifer.java` — aquifer fluid placement
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/OreVeinifier.java` — ore vein generation
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/SurfaceRules.java` — surface rule DSL
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/SurfaceSystem.java` — surface rule application
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/Heightmap.java` — heightmap types and computation
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/Beardifier.java` — structure terrain adjustment
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/BelowZeroRetrogen.java` — below-zero retrogen
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/RandomState.java` — per-seed random state
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/WorldDimensions.java` — dimension selection
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/WorldGenSettings.java` — worldgen options
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/WorldgenRandom.java` — seeded random wrappers
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/LegacyRandomSource.java` — Java LCG random
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/XoroshiroRandomSource.java` — xoroshiro128++ random
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/Noises.java` — noise parameter registry
- `decompiled-server-26.1.2/net/minecraft/world/level/chunk/status/ChunkStatusTasks.java` — chunk status task bodies
- `decompiled-server-26.1.2/net/minecraft/world/level/chunk/ChunkGenerator.java` — generator interface
- `decompiled-server-26.1.2/net/minecraft/world/level/chunk/ChunkAccess.java` — chunk data model
- `decompiled-server-26.1.2/net/minecraft/world/level/chunk/ProtoChunk.java` — in-progress chunk
- `decompiled-server-26.1.2/net/minecraft/world/level/chunk/LevelChunk.java` — loaded chunk
- `decompiled-server-26.1.2/net/minecraft/world/level/chunk/storage/SerializableChunkData.java` — chunk NBT format
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/` — feature implementations
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/` — feature configs
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/` — placement modifiers
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/carver/` — carver implementations
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/` — structure generation
- `decompiled-server-26.1.2/net/minecraft/world/level/biome/` — biome source implementations
- `decompiled-server-26.1.2/data/minecraft/worldgen/` — all vanilla worldgen data JSON
- `decompiled-server-26.1.2/net/minecraft/data/worldgen/` — bootstrap code for worldgen data
- `RustCraft/src/worldgen.rs` — RustCraft worldgen implementation
- `RustCraft/src/worldgen_comparison.rs` — worldgen comparison tests
- `RustCraft/src/biome.rs` — RustCraft biome implementation

## Current Status Note

The live RustCraft spawn terrain is **synthetic scaffolding** (deterministic noise-based preview terrain), not vanilla world generation. All items below represent the gap between the current scaffolding and full vanilla worldgen parity.

## World Presets and Dimension Loading

- [ ] Implement world-preset and dimension-stem loading from registry data (`data/minecraft/worldgen/world_preset/`, `data/minecraft/worldgen/flat_level_generator_preset/`) rather than hard-coded Rust defaults
- [ ] Implement `WorldDimensions` codec: select overworld/nether/end generator from registry entries
- [ ] Implement `WorldGenSettings`: seed, `generateStructures`, `generateBonusChest`, dimension map
- [ ] Implement `LevelStem` codec pairing `DimensionType` holder with `ChunkGenerator`
- [ ] Add test: `level-type=flat`, `level-type=amplified`, `level-type=large_biomes`, `level-type=single_biome_surface`, custom world preset all select correct generators

## Random Source Parity

- [ ] Implement `LegacyRandomSource`: bit-for-bit Java `java.util.Random` LCG (`seed = (seed * 0x5DEECE66DL + 0xBL) & ((1L << 48) - 1)`) for legacy worldgen paths
- [ ] Implement `XoroshiroRandomSource` / `Xoroshiro128PlusPlus`: exact algorithm from `WorldgenRandom`
- [ ] Implement `WorldgenRandom` wrappers: `setDecorationSeed(baseSeed, x, z)`, `setFeatureSeed(baseSeed, counter, step)`, `setLargeFeatureSeed(baseSeed, x, z)`, `setLargeFeatureWithSalt(baseSeed, x, z, salt)`
- [ ] Implement `PositionalRandomFactory`: `fromHashOf(string)` for named noise forks
- [ ] Implement `RandomState`: per-level seed state, `getOrCreateRandomFactory(noise)`, `legacyLevelSeed()`
- [ ] Add byte-for-byte parity test: legacy random and xoroshiro produce identical values to Java for same seed/sequence

## Biome Sources and Selection

- [ ] Implement `FixedBiomeSource`: returns single biome everywhere (for single-biome preset)
- [ ] Implement `CheckerboardBiomeSource`: alternating biomes in checkerboard pattern
- [ ] Implement `MultiNoiseBiomeSource`: overworld multi-noise (uses climate sampler and R-tree parameter matching), nether multi-noise (small parameter set)
- [ ] Implement `TheEndBiomeSource`: island-based biome logic (the_end center, end_highlands, end_midlands, end_barrens, small_end_islands)
- [ ] Implement `Climate.Sampler` and R-tree (`Climate.RTree`) parameter matching for overworld biome assignment
- [ ] Implement climate point evaluation via noise router outputs: temperature, humidity, continentalness, erosion, depth, weirdness, factor
- [ ] Implement `MultiNoiseBiomeSourceParameterList` for overworld and nether parameter sets from vanilla `data/minecraft/worldgen/multi_noise_biome_source_parameter_list/`
- [ ] Add parity test: biome at (0,64,0) overworld matches vanilla for representative seeds
- [ ] Add parity test: nether and end biome assignment matches vanilla for representative coordinates

## Biome Generation Settings and Mob Spawns

- [ ] Implement `BiomeGenerationSettings` loading from registry data: carvers list, features list (per GenerationStep)
- [ ] Implement `MobSpawnSettings` loading: spawner data (entity type, weight, min/max group size), spawn costs (energy budget, charge), creature probability
- [ ] Add test: loaded biome generation settings for `minecraft:plains` match vanilla feature step counts and carver inclusion

## Chunk Status Pipeline

- [ ] Implement chunk status pipeline in vanilla order (from `ChunkStatus`): EMPTY → STRUCTURE_STARTS → STRUCTURE_REFERENCES → BIOMES → NOISE → SURFACE → CARVERS → FEATURES → INITIALIZE_LIGHT → LIGHT → SPAWN → FULL
- [ ] Implement each status with correct region dependency sizes (e.g., FEATURES needs 8-radius neighbor chunks, SURFACE needs 0)
- [ ] Implement `ChunkStatusTasks` task bodies for each status
- [ ] Implement `ChunkGenerator` generator methods invoked per status: `createBiomes()`, `fillFromNoise()`, `buildSurface()`, `applyCarvers()`, `applyBiomeDecoration()`, `createStructures()`, `createReferences()`, `spawnOriginalMobs()`
- [ ] Route live join-time chunk batch through generated `LevelChunk` output and `ClientboundLevelChunkWithLightPacket::from_chunk` serializer (replacing the current synthetic chunk writer)
- [ ] Add parity test: chunk at overworld (0,0) reaches FULL status with correct section count and heightmap entries

## Noise Settings and Router

- [ ] Implement `NoiseGeneratorSettings` codec loading from `data/minecraft/worldgen/noise_settings/`
- [ ] Implement `NoiseRouter` with named output channels: `barrierNoise`, `fluidLevelFloodednessNoise`, `fluidLevelSpreadNoise`, `lavaNoise`, `temperature`, `vegetation`, `continentalness`, `erosion`, `depth`, `ridges`, `initialDensityWithoutJaggedness`, `finalDensity`, `veinToggle`, `veinRidged`, `veinGap`
- [ ] Implement `NoiseSettings` (min Y, height, sampling noise scale, noise size XZ/Y)
- [ ] Implement `RandomState` noise fork caching: `getOrCreateNoise(ResourceKey<NormalNoise.NoiseParameters>)`
- [ ] Add test: noise settings for overworld match vanilla min Y = -64, height = 384

## Density Functions

- [ ] Implement all density function codecs and runtime evaluation (`DensityFunctions`):
  - [ ] `BlendAlpha`, `BlendOffset`, `BlendDensity`
  - [ ] `Clamp`, `Mapped`, `Abs`, `Square`, `Cube`, `HalfNegative`, `QuarterNegative`, `Squeeze`
  - [ ] `Add`, `Mul`, `Min`, `Max`
  - [ ] `Spline` (with `CubicSpline` evaluation)
  - [ ] `YClampedGradient`
  - [ ] `InterpolatedNoise` (trilinear interpolation of NormalNoise)
  - [ ] `ShiftedNoise`
  - [ ] `Noise` (direct NormalNoise sample)
  - [ ] `WeirdScaledSampler`
  - [ ] `OldBlendedNoise`
  - [ ] `ShiftA`, `ShiftB`, `Shift` (noise-based position offsets)
  - [ ] `Constant`
  - [ ] `Cache2D`, `CacheFlat`, `CacheOnce`, `CacheAllInCell` (memoization)
  - [ ] `FlatCache`, `EndIslands`
  - [ ] `Beardifier` (structure terrain bump)
  - [ ] Marker wrappers: `Interpolated`, `FlatCached`
  - [ ] `HolderHolder` (registry-backed density function reference)
- [ ] Implement `NoiseChunk` cell-based sampling loop with proper XZ/Y cell sizing from noise settings
- [ ] Add parity test: `finalDensity` at overworld (0,100,0) matches vanilla output for seed 0

## Noise Samplers

- [ ] Implement `NormalNoise` (octave Perlin): exact Java `PerlinNoiseSampler` per octave, amplitude scaling from persistence
- [ ] Implement `PerlinNoise` (simplex-like, pre-1.18 compat path) for legacy random path
- [ ] Implement `BlendedNoise` (old 3D noise for `OldBlendedNoise` density function)
- [ ] Implement `SimplexNoise` for end terrain and end island generation
- [ ] Add byte-for-byte parity test: `NormalNoise` at specific coordinates matches Java output for same seed

## Aquifer and Fluid Placement

- [ ] Implement `Aquifer.NoiseBasedAquifer`: barrier noise, fluid-level flood noise, fluid-level spread noise, lava noise; place water/lava aquifer pockets based on pressure differential
- [ ] Implement `Aquifer.FluidPicker`: `globalFluidPicker` for bedrock lava floor (y < minY+10) and sea-level water
- [ ] Implement `AquiferStatus`: fluid type and y-level for each aquifer cell
- [ ] Implement `update-fluid` propagation flags on `NoiseChunk` for carved/noise-filled terrain
- [ ] Add parity test: aquifer fluid at y=-55 below sea level matches vanilla fluid type placement

## Ore Veinifier

- [ ] Implement `OreVeinifier`: copper/iron ore veins, raw ore block placement, gap noise, positional random decisions, `VeinType` (COPPER above 0, IRON below 0)
- [ ] Add parity test: ore vein presence at coordinates known to contain veins for a fixed seed

## Surface Rules and Material Rules

- [ ] Implement `SurfaceRules` DSL codec: `Sequence`, `Condition`, `Block`, `Bandlands`, `StonDepthCheck`, `AbovePreliminarySurface`, `BiomeCondition`, `NoiseThreshold`, `VerticalGradient`, `YAbove`, `Water`, `Steep`, `Hole`, `SurfaceType` (FLOOR/CEILING/FLOOR_ONLY)
- [ ] Implement `SurfaceSystem.buildSurface()`: iterate each chunk column, evaluate surface rule tree top-down, place surface blocks (grass, dirt, gravel, sand, etc.) per biome
- [ ] Implement surface rule data loading from `data/minecraft/worldgen/noise_settings/<name>.json` `surface_rule` field
- [ ] Add parity test: plains biome at y=64 surface has grass block, y=63 has dirt, stone below matches vanilla for seed 0

## Carvers

- [ ] Implement carver codec loading from `data/minecraft/worldgen/configured_carver/`
- [ ] Implement `CaveWorldCarver`: cave tunnel carving with sphere-widening, floor placement, lava pool at depth
- [ ] Implement `CanyonWorldCarver`: canyon trench carving with angled ceiling
- [ ] Implement `NetherWorldCarver`: nether cave variants
- [ ] Implement carver execution during `ChunkGenerator.applyCarvers()`: AIR/LIQUID carving step, mask tracking to avoid double-carving
- [ ] Add parity test: cave opening at overworld coordinates known to have caves for a fixed seed

## Placed Features and Decoration

- [ ] Implement `PlacedFeature` evaluation: load placement modifiers, evaluate `PlacementModifier` list in order, for each valid position invoke `ConfiguredFeature.place()`
- [ ] Implement all `PlacementModifier` types: `CountPlacement`, `RarityFilter`, `InSquarePlacement`, `HeightRangePlacement`, `HeightmapPlacement`, `BiomeFilter`, `SurfaceRelativeThresholdFilter`, `EnvironmentScanPlacement`, `RandomOffsetPlacement`, `CarvingMaskPlacement`, `CountOnEveryLayerPlacement`, `NoiseThresholdCountPlacement`, `NoiseBasedCountPlacement`
- [ ] Implement feature support primitives: `HeightProvider`, `VerticalAnchor`, `BlockPredicate`, `StateProvider` (rotated, simple, weighted), `FeatureSize` (two-layers, three-layers), trunk/foliage/root placers, tree decorators
- [ ] Implement `FeatureSorter`: deterministic decoration ordering per biome step; seed derivation `baseSeed + featureIndex + biomeFeatureCounter`
- [ ] Implement biome decoration ordering: `GenerationStep.Decoration` ordering (RAW_GENERATION → LAKES → LOCAL_MODIFICATIONS → UNDERGROUND_STRUCTURES → SURFACE_STRUCTURES → STRONGHOLDS → UNDERGROUND_ORES → UNDERGROUND_DECORATION → FLUID_SPRINGS → VEGETAL_DECORATION → TOP_LAYER_MODIFICATION)
- [ ] Implement `feature-cycle` error behavior: throw exception on infinite feature recursion
- [ ] Add parity test: decoration seed at biome/chunk/step matches vanilla for a fixed overworld seed

## Individual Feature Implementations

- [ ] Implement tree features: OakFeature, BirchFeature, JungleTreeFeature, AcaciaFeature, DarkOakFeature, SpruceFeature, CherryTreeFeature, MangroveTreeFeature; all trunk/foliage placers with correct randomness
- [ ] Implement vegetation features: grass patch, flower patch, mushroom, huge mushroom, bamboo, kelp, sea grass, coral, jungle bush, nether fungi, twisting/weeping vines, small dripleaf, big dripleaf, cave vines, pointed dripstone cluster, dripstone room
- [ ] Implement ore features: `OreFeature`, `ScatteredOreFeature`, ore placement configs for all vanilla ores
- [ ] Implement spring features: water/lava spring pocket
- [ ] Implement lake features: water/lava lake (now placed rarely)
- [ ] Implement disk features: gravel/sand/clay disk
- [ ] Implement geode feature: amethyst geode with cracked/budding/smooth layers
- [ ] Implement fossil feature: bone block/coal ore fossil
- [ ] Implement monster room feature: dungeon room with spawner and chests
- [ ] Implement fill-layer, simple-block, scattered-raw-ore, delta-feature (basalt delta), basalt-columns, end-spike, end-island, end-gateway, chorus plant, blue-ice, glowstone-blob, multiface-growth (glow lichen / dripleaf), ice-spike, ice-patch, freeze-top-layer features
- [ ] Add parity test: ore vein positions at known coordinates for fixed seed match vanilla

## Structure Generation

- [ ] Implement `StructureSet` placement: `ConcentricRingsStructurePlacement` (strongholds), `RandomSpreadStructurePlacement` (villages, outposts, etc.)
- [ ] Implement structure-check parity: `StructureManager.getStructureAt()`, starts vs. references, `structuresBelowEverything` flag
- [ ] Implement `Beardifier.forStructure()`: terrain adjustment for structures via density function integration
- [ ] Implement structure start generation: `Structure.generate()`, bounding box, piece container, `StructureStart.placeInChunk()`, `JigsawStructure.generatePieces()`

### Non-Jigsaw Structure Pieces
- [ ] Implement `MineshaftPieces`: corridor, cross, staircase, room; supported/gravel fill; chest loot
- [ ] Implement `StrongholdPieces`: 5×5 room maze generator, portal room, library, dead-end, chest, prison, crossing, straight, staircase types
- [ ] Implement `OceanMonumentPieces`: monument room layout and fill
- [ ] Implement `WoodlandMansionPieces`: mansion wing/room layout
- [ ] Implement `NetherFortressPieces`: corridor, staircase, crossing, bridge
- [ ] Implement `JungleTemplePieces`: temple room with traps and loot
- [ ] Implement `DesertPyramidPieces`: pyramid room with trap TNT and loot
- [ ] Implement `SwampHutPiece`: witch's hut
- [ ] Implement `IglooPieces`: igloo room, optional basement with zombie villager
- [ ] Implement `ShipwreckPieces`: multiple variants from NBT templates
- [ ] Implement `OceanRuinPieces`: large/small variants from NBT templates
- [ ] Implement `BuriedTreasurePiece`: single chest at mapped coordinate
- [ ] Implement `RuinedPortalPiece`: multiple NBT variants with optional lava pool
- [ ] Implement `NetherFossilPiece`: bone-block fossil NBT variants
- [ ] Implement `EndCityPieces`: end city NBT template rooms and ship
- [ ] Add parity test: stronghold portal room position matches vanilla `/locate` output for fixed seed

### Jigsaw Structure Pieces
- [ ] Implement `JigsawPlacement`: breadth-first jigsaw placement, depth limit, joint type (aligned/rollable), projection (rigid/terrain_facing), `PoolAliasLookup`
- [ ] Implement `SinglePoolElement`, `FeaturePoolElement`, `LegacySinglePoolElement`, `ListPoolElement`, `EmptyPoolElement`
- [ ] Implement structure processor list: `BlockIgnoreProcessor`, `BlockRotProcessor`, `BlockStateMatchProcessor`, `BlockMatchProcessor`, `GravityProcessor`, `JigsawReplacementProcessor`, `LavaSubmergedBlockProcessor`, `NopProcessor`, `ProtectedBlocksProcessor`, `RuleProcessor`, `CappedProcessor`
- [ ] Implement template pool loading from `data/minecraft/worldgen/template_pool/`
- [ ] Implement jigsaw content for villages, pillager outposts, bastions, ancient cities, trail ruins, and trial chambers from template pools and NBT structure files
- [ ] Add parity test: village structure at known coordinates for fixed seed has correct bounding box

## Terrain Blending and Retrogen

- [ ] Implement `BelowZeroRetrogen`: upgrade old pre-1.18 chunks to include below-Y=0 terrain, carve mask blending
- [ ] Implement terrain blending for old/new chunk boundaries: `BlendingData`, `Blender.getBlendingDataValue()`, blend alpha/offset density functions applied at old chunk edges
- [ ] Keep chunk data fields compatible with blending from initial implementation (never skip `BlendingData` NBT fields)

## Spawn Position Parity

- [ ] Implement `NoiseBasedChunkGenerator.findSpawnPosition()` using climate spawn targets and heightmap valid-block checks
- [ ] Implement server-level default spawn persistence in `level.dat` (`SpawnX`, `SpawnY`, `SpawnZ`, `SpawnAngle`)
- [ ] Implement initial chunk readiness check before spawn position is considered safe
- [ ] Add parity test: first-join spawn position matches vanilla for fixed seed (within vanilla's randomized search radius)

## Chunk Generation Mob Spawns

- [ ] Implement `ChunkGenerator.spawnOriginalMobs()`: spawns mobs during SPAWN status for applicable structures (monster rooms, etc.)
- [ ] Implement generated chunk structure mob spawns separately from tick-time natural spawning

## Generated Chunk Output Completeness

- [ ] Implement `ProtoChunk` with: block sections, biome sections, heightmaps (MOTION_BLOCKING, MOTION_BLOCKING_NO_LEAVES, OCEAN_FLOOR, WORLD_SURFACE), carving masks (AIR/LIQUID), block entities queue, entity generation queue, structures (starts + references), ticks (block + fluid), lights (sky + block), post-processing markers, inhabited time, status field
- [ ] Implement lighting handoff for generated chunks: sky/block light arrays match vanilla expectations after terrain/carvers/features run before LIGHT status
- [ ] Implement region/chunk persistence compatibility: serialize generated `LevelChunk` to vanilla 26.1.2 `SerializableChunkData` NBT format with `DataVersion`, `xPos`, `zPos`, `Status`, `sections`, `block_entities`, `structures`, `Heightmaps`, `blending_data`, `below_zero_retrogen` fields

## Worldgen Parity Testing

- [ ] Build a Java-guided worldgen trace harness: run official `server.jar` for fixed seeds/chunks, record chunk status transitions, biome IDs, heightmaps, section palettes, structures, feature counts, and serialized chunk NBT
- [ ] Add deterministic parity fixtures for overworld chunks at (0,0), (1,0), (0,1), (16,16) across seeds 0, 1, -1, and a large prime for each: biome IDs, heightmap values, representative block positions
- [ ] Add deterministic parity fixtures for nether chunks and end chunks across multiple seeds
- [ ] Add progressive acceptance gates: real flat generator first → real noise terrain without decoration → surfaces/carvers → biome decoration → structures → full persistence/lighting parity
- [ ] Add worldgen comparison regression tests that fail when RustCraft output diverges from a previously accepted vanilla snapshot
