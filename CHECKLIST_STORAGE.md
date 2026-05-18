# World Storage, NBT, and DataFix Checklist

## Relevant Java Source Files

- `decompiled-server-26.1.2/net/minecraft/world/level/storage/LevelStorageSource.java` — world folder access
- `decompiled-server-26.1.2/net/minecraft/world/level/storage/PrimaryLevelData.java` — level.dat main data
- `decompiled-server-26.1.2/net/minecraft/world/level/storage/DerivedLevelData.java` — dimension-derived data
- `decompiled-server-26.1.2/net/minecraft/world/level/storage/ServerLevelData.java` — server level data interface
- `decompiled-server-26.1.2/net/minecraft/world/level/storage/WorldData.java` — world data interface
- `decompiled-server-26.1.2/net/minecraft/world/level/storage/LevelSummary.java` — world list summary
- `decompiled-server-26.1.2/net/minecraft/world/level/storage/LevelVersion.java` — version info in level.dat
- `decompiled-server-26.1.2/net/minecraft/world/level/storage/PlayerDataStorage.java` — playerdata files
- `decompiled-server-26.1.2/net/minecraft/world/level/storage/CommandStorage.java` — command storage data
- `decompiled-server-26.1.2/net/minecraft/world/level/saveddata/SavedDataStorage.java` — saved data containers
- `decompiled-server-26.1.2/net/minecraft/nbt/` — NBT codec and IO
- `decompiled-server-26.1.2/net/minecraft/util/datafix/` — DataFixer schemas and fixes
- `decompiled-server-26.1.2/net/minecraft/world/level/chunk/storage/SerializableChunkData.java` — chunk NBT
- `decompiled-server-26.1.2/net/minecraft/world/level/chunk/storage/RegionFile.java` — region file format
- `RustCraft/src/storage/` — RustCraft storage modules
- `RustCraft/src/storage/nbt.rs` — RustCraft NBT
- `RustCraft/src/storage/region.rs` — RustCraft region file
- `RustCraft/src/storage/datafix.rs` — RustCraft datafix
- `RustCraft/src/storage/world.rs` — RustCraft world storage
- `RustCraft/src/storage/chunk.rs` — RustCraft chunk storage

## Level Storage and Session Management

- [ ] Implement `LevelStorageSource`: world-folder enumeration, lock-file acquisition (`session.lock`), world-path resolution for dimensions, backup, delete, rename, and validate
- [ ] Implement `LevelStorageAccess`: per-session interface with `getDimensionPath()`, `readLevelData()`, `saveLevelData()`, `deleteLevel()`, dimension folder layout (`DIM-1/`, `DIM1/`, named dimensions under `dimensions/`)
- [ ] Implement `PrimaryLevelData`: all `level.dat` fields including `DataVersion`, `Version` (id/name/series/snapshot), `LevelName`, `SpawnX/Y/Z/Angle`, `GameType`, `Difficulty`, `DayTime`, `Time`, `generatorName`/`generatorSettings` → `WorldGenSettings` codec, `allowCommands`, `hardcore`, `initialized`, `WasModded`, `DataPacks` (enabled/disabled lists), `ScheduledEvents` (`TimerQueue`), `ServerBrands`, `CustomBossEvents`, `DragonFight`, `scoreboard`, `GameRules`
- [ ] Implement `DerivedLevelData`: thin view over `PrimaryLevelData` for individual dimensions, inheriting game rules and world properties but with per-dimension spawn/seed
- [ ] Implement `LevelVersion`: extracting `DataVersion`, world series/snapshot from `level.dat` before full load for upgrade check
- [ ] Add parity test: `level.dat` written and re-read with same fields for a default world vs. vanilla-generated `level.dat`

## Player Data Storage

- [ ] Implement `PlayerDataStorage`: save compressed gzip `<uuid>.dat`, rotate to `<uuid>.dat_old` on save, on load corruption copy to `<uuid>_corrupted_<timestamp>.dat` and fall back to `<uuid>.dat_old`
- [ ] Implement player NBT fields: `Pos` (3 doubles), `Rotation` (2 floats), `Motion` (3 doubles), `Health` (float), `FoodLevel` (int), `FoodSaturationLevel` (float), `FoodExhaustion` (float), `XpP` (float), `XpLevel` (int), `XpTotal` (int), `XpSeed` (int), `Score` (int), `SelectedItemSlot` (int, 0–8), `Inventory` (list of slot NBT), `EnderItems` (ender chest), `playerGameType` (int), `previousPlayerGameType` (int), `SpawnX/Y/Z`, `SpawnForced` (bool), `SpawnDimension`, `seenCredits` (bool), `recipeBook` (compound), `LastDeathLocation` (optional dimension+pos), `enteredNetherPosition` (optional), `RootVehicle` (optional), `abilities` compound, `active_effects` list
- [ ] Implement player data bounds clamping on load: `Health` clamped to [0, max-health], `FoodLevel` to [0, 20], `SelectedItemSlot` to [0, 8], invalid `playerGameType` to 0 (survival)
- [ ] Add parity test: player NBT save/load round-trip preserves all fields identically for a full-state profile
- [ ] Add Mineflayer playerdata round-trip test: change position, rotation, inventory, selected slot, health, food, XP, game mode, recipe book, stats; disconnect; reconnect; verify all fields restored

## Saved Data Storage

- [ ] Implement `SavedDataStorage`: per-level `.dat` files under `data/` folder, `get(key)` loads, `computeIfAbsent(key, factory)` creates on miss, `set(key, data)` marks dirty, autosave on level save
- [ ] Implement `CommandStorage`: `minecraft:` and function-namespaced NBT storage, `/data storage` command access
- [ ] Implement tag value helpers: `TagValueInput`, `TagValueOutput` for NBT-backed data
- [ ] Add parity test: command storage NBT written by `/data merge storage` command is readable by `/data get storage`

## DataFixer Strategy

- [ ] Implement a DataFixer strategy covering the following schema families (version ranges from pre-1.9 through 26.1.2 DataVersion):
  - [ ] Block ID renames (numeric → namespaced, block state flattening)
  - [ ] Block entity type renames and field migrations
  - [ ] Entity type renames and field migrations
  - [ ] Item ID renames (numeric → namespaced, stack meta flattening)
  - [ ] Chunk format upgrades (pre-Anvil → Anvil → 1.16+ format → 1.18+ sections format)
  - [ ] POI file creation from existing chunk data
  - [ ] Advancements and stats file format changes
  - [ ] Scoreboard data format changes
  - [ ] Structure data format changes (NBT template migration)
  - [ ] Text component format changes (JSON string → compound)
  - [ ] Villager data migrations (profession IDs, trade format)
  - [ ] WorldGen settings format changes (pre-1.16 generator → new WorldGenSettings codec)
  - [ ] Versioned registry renames across DataVersions
- [ ] Implement `DataVersion` tracking: embed `DataVersion` int in all saved files, detect version mismatch on load
- [ ] Implement upgrade CLI path: `--forceUpgrade` flag triggers DataFixer pass on all chunks and entities
- [ ] Add parity test: a vanilla 1.20.x world loaded by the rebuilt server upgrades without data loss for representative blocks/entities/players
- [ ] Add parity test: `--eraseCache` removes only the cache data without corrupting world content

## Operational Files

- [ ] Implement all operator-facing files: `eula.txt`, `server.properties`, `ops.json`, `whitelist.json`, `banned-players.json`, `banned-ips.json`, `usercache.json` with vanilla-compatible JSON schemas and field names
- [ ] Implement `session.lock`: written on world open, exclusive lock enforced, released on clean shutdown; startup refuses if lock held by another process
- [ ] Implement `level.dat` + `level.dat_old` rotation: write new `level.dat` atomically (temp file + rename), keep previous as `level.dat_old`
- [ ] Implement region file compression: supports both `zlib` (type 2) and `lz4` (type 4, if `region-file-compression=lz4`) in chunk headers
- [ ] Implement entity region files: `<dim>/entities/*.mca` separate from block region files
- [ ] Implement POI region files: `<dim>/poi/*.mca` with POI type and occupation counts
- [ ] Implement playerdata: `playerdata/<uuid>.dat` and `playerdata/<uuid>.dat_old`
- [ ] Implement advancements: `advancements/<uuid>.json`
- [ ] Implement stats: `stats/<uuid>.json`
- [ ] Implement server icon: `server-icon.png` (64×64 PNG, base64-encoded for status response)
- [ ] Implement crash reports: `crash-reports/crash-<timestamp>-server.txt`
- [ ] Implement logs rotation: `logs/latest.log` + `logs/<date>-<n>.log.gz`
- [ ] Implement generated reports: `generated/` directory for registry/command/tag reports from `--report` flag

## Management Server (JSON-RPC)

- [ ] Implement JSON-RPC management server: method dispatch, JSON schema validation for parameters and results
- [ ] Implement management server methods: `minecraft:list_players`, `minecraft:kick_player`, `minecraft:ban_player`, `minecraft:pardon_player`, etc. (all methods from decompiled `management-server.json` schema)
- [ ] Implement management server outgoing notifications: `minecraft:player_joined`, `minecraft:player_left`
- [ ] Implement player DTOs in management server responses: UUID, name, latency, game mode
- [ ] Implement pending request tracking, response correlation
- [ ] Implement allowed origins CORS check for WebSocket management connections
- [ ] Implement TLS support for management server connections
- [ ] Implement shutdown behavior: in-flight management requests are completed or rejected on shutdown

## Game Test Framework Hooks

- [ ] Implement game test framework hooks sufficient for parity test execution, or document a replacement harness that provides the same test coverage
- [ ] Implement `TestBlockEntity`, `TestInstanceBlockEntity`, `/test` command stubs needed by vanilla test infrastructure if any tests rely on them

## Generated Data / Report Tooling

- [ ] Implement `--report` flag: generate `generated/reports/` containing `registries.json` (all registry IDs), `commands.json` (Brigadier tree), `biomes.json`, `blocks.json`, `items.json`, `tags/` (all tag files)
- [ ] Add parity test: `--report` output registry IDs and counts match official `server.jar --report` output for vanilla 26.1.2
