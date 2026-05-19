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

- [x] Implement `LevelStorageSource`: world-folder enumeration, lock-file acquisition (`session.lock`), world-path resolution for dimensions, backup, delete, rename, and validate - `storage::world::LevelStorageSource` mirrors Java 26.1.2 world candidate discovery, level id validation, symlink validation, per-world session locking, access creation, backup directory selection, and Anvil naming.
- [x] Implement `LevelStorageAccess`: per-session interface with `getDimensionPath()`, `readLevelData()`, `saveLevelData()`, `deleteLevel()`, dimension folder layout - `storage::world::LevelStorageAccess` wraps `WorldLayout` with an exclusive `session.lock`, level data read/write through `level.dat` + `level.dat_old`, Java 26.1.2 dimension folders under `dimensions/<namespace>/<path>`, level rename, backup copy excluding `session.lock`, and locked delete.
- [ ] Implement `PrimaryLevelData`: all `level.dat` fields including `DataVersion`, `Version` (id/name/series/snapshot), `LevelName`, `SpawnX/Y/Z/Angle`, `GameType`, `Difficulty`, `DayTime`, `Time`, `generatorName`/`generatorSettings` → `WorldGenSettings` codec, `allowCommands`, `hardcore`, `initialized`, `WasModded`, `DataPacks` (enabled/disabled lists), `ScheduledEvents` (`TimerQueue`), `ServerBrands`, `CustomBossEvents`, `DragonFight`, `scoreboard`, `GameRules`
- [x] Implement `DerivedLevelData`: thin view over `PrimaryLevelData` for individual dimensions, inheriting game rules and world properties but with per-dimension spawn/seed - `storage::world::DerivedLevelData` follows Java 26.1.2 delegation: world name, game type, hardcore, allow-commands, difficulty, and difficulty lock come from world data; respawn data, game time, initialization, and dimension seed come from wrapped server-level data; Java no-op setters remain no-ops.
- [x] Implement `LevelVersion`: extracting `DataVersion`, world series/snapshot from `level.dat` before full load for upgrade check
- [ ] Add parity test: `level.dat` written and re-read with same fields for a default world vs. vanilla-generated `level.dat`

## Player Data Storage

- [x] Implement `PlayerDataStorage`: save compressed gzip `<uuid>.dat`, rotate to `<uuid>.dat_old` on save, on load corruption copy to `<uuid>_corrupted_<timestamp>.dat` and fall back to `<uuid>.dat_old`
- [ ] Implement player NBT fields: `Pos` (3 doubles), `Rotation` (2 floats), `Motion` (3 doubles), `Health` (float), `FoodLevel` (int), `FoodSaturationLevel` (float), `FoodExhaustion` (float), `XpP` (float), `XpLevel` (int), `XpTotal` (int), `XpSeed` (int), `Score` (int), `SelectedItemSlot` (int, 0–8), `Inventory` (list of slot NBT), `EnderItems` (ender chest), `playerGameType` (int), `previousPlayerGameType` (int), `SpawnX/Y/Z`, `SpawnForced` (bool), `SpawnDimension`, `seenCredits` (bool), `recipeBook` (compound), `LastDeathLocation` (optional dimension+pos), `enteredNetherPosition` (optional), `RootVehicle` (optional), `abilities` compound, `active_effects` list
- [x] Implement player data bounds clamping on load: `Health` clamped to [0, max-health], `FoodLevel` to [0, 20], `SelectedItemSlot` to [0, 8], invalid `playerGameType` to 0 (survival) — `play_session_state_from_nbt_clamps_vanilla_playerdata_bounds` plus raw 26.1.2 fallback tests cover login packet effects
- [ ] Add parity test: player NBT save/load round-trip preserves all fields identically for a full-state profile
- [x] Add Mineflayer playerdata round-trip test: change position, rotation, inventory, selected slot, health, food, XP, game mode, recipe book, stats; disconnect; reconnect; verify all fields restored

## Saved Data Storage

- [x] Implement `SavedDataStorage`: per-level `.dat` files under `data/` folder, `get(key)` loads, `computeIfAbsent(key, factory)` creates on miss, `set(key, data)` marks dirty, autosave on level save — `storage::saved_data::SavedDataStorage` caches loaded data, wraps saves in `{data,DataVersion}`, writes gzip NBT under namespaced data paths, and clears dirty state after save
- [x] Implement `CommandStorage`: `minecraft:` and function-namespaced NBT storage, `/data storage` command access — `storage::saved_data::CommandStorage` stores per-namespace `command_storage.dat` containers, removes empty compounds, lists keys, and persists through `SavedDataStorage`
- [x] Implement tag value helpers: `TagValueInput`, `TagValueOutput` for NBT-backed data — `storage::tag_value` wraps Rust NBT compounds with vanilla-style typed reads/writes, child compounds, compound lists, discard/replacement, and mismatch problem reporting
- [x] Add parity test: command storage NBT written by `/data merge storage` command is readable by `/data get storage` — `command_storage_data_merge_then_get_matches_java_storage_accessor` exercises Java-equivalent `StorageDataAccessor` semantics: recursive compound merge, unchanged-merge detection, invalid non-compound rejection, then retrieval through `data_get_storage`

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

- [x] Implement all operator-facing files: `eula.txt`, `server.properties`, `ops.json`, `whitelist.json`, `banned-players.json`, `banned-ips.json`, `usercache.json` with vanilla-compatible JSON schemas and field names
- [x] Implement `session.lock`: written on world open, exclusive lock enforced, released on clean shutdown; startup refuses if lock held by another process
- [x] Implement `level.dat` + `level.dat_old` rotation: write new `level.dat` atomically (temp file + rename), keep previous as `level.dat_old`
- [x] Implement region file compression: supports both `zlib` (type 2) and `lz4` (type 4, if `region-file-compression=lz4`) in chunk headers
- [x] Implement entity region files: `<dim>/entities/*.mca` separate from block region files
- [x] Implement POI region files: `<dim>/poi/*.mca` with POI type and occupation counts
- [x] Implement playerdata: `playerdata/<uuid>.dat` and `playerdata/<uuid>.dat_old`
- [x] Implement advancements: `advancements/<uuid>.json`
- [x] Implement stats: `stats/<uuid>.json`
- [x] Implement server icon: `server-icon.png` (64×64 PNG, base64-encoded for status response)
- [x] Implement crash reports: `crash-reports/crash-<timestamp>-server.txt`
- [x] Implement logs rotation: `logs/latest.log` + `logs/<date>-<n>.log.gz`
- [x] Implement generated reports: `generated/` directory for registry/command/tag reports from `--report` flag — `generated_reports::generate_reports()` writes vanilla-named report and tag outputs, and `main::tests::report_flag_generates_reports_and_exits_before_eula_gate` covers CLI wiring

## Management Server (JSON-RPC)

- [ ] Implement JSON-RPC management server: method dispatch, JSON schema validation for parameters and results
- [ ] Implement management server methods: `minecraft:list_players`, `minecraft:kick_player`, `minecraft:ban_player`, `minecraft:pardon_player`, etc. (all methods from decompiled `management-server.json` schema)
- [x] Implement management server outgoing notifications: vanilla `players/joined`, `players/left`, and the full `OutgoingRpcMethods` set — `OutgoingNotification`/`OUTGOING_METHODS` mirror the decompiled method names and queue broadcasts per connected client
- [x] Implement player DTOs in management server responses: UUID, name, latency, game mode — `PlayerDto` now serializes `id`, `name`, `latency`, and `gameMode`, with discovery schema coverage
- [x] Implement pending request tracking, response correlation — `ManagementServerState::handle_client_request()` records pending JSON-RPC IDs per client, correlates responses by ID, and clears abandoned requests on disconnect
- [x] Implement allowed origins CORS check for WebSocket management connections — `AllowedOrigins::parse()` and `authorize_request()` enforce exact, wildcard, and empty-origin policies before bearer-secret authentication
- [ ] Implement TLS support for management server connections
- [x] Implement shutdown behavior: in-flight management requests are completed or rejected on shutdown — `ManagementServerState::shutdown()` queues vanilla `server/stopping`, rejects tracked pending requests, and clears connection/request state

## Game Test Framework Hooks

- [x] Implement game test framework hooks sufficient for parity test execution, or document a replacement harness that provides the same test coverage
- [x] Implement `TestBlockEntity`, `TestInstanceBlockEntity`, `/test` command stubs needed by vanilla test infrastructure if any tests rely on them — `block_entity.rs` includes test block/test instance block entity type registration plus Java-field state stubs for mode/message/powered, test instance data/status/errors, save/load, and basic trigger/status transitions

## Generated Data / Report Tooling

- [x] Implement `--report` flag: generate `generated/reports/` containing `registries.json` (all registry IDs), `commands.json` (Brigadier tree), `biomes.json`, `blocks.json`, `items.json`, `tags/` (all tag files): `generated_reports::generate_reports()` writes the vanilla-named report files plus generated tag files, and `main::run()` exits through the report path before EULA gating; covered by `report_generator_writes_vanilla_named_outputs` and `report_flag_generates_reports_and_exits_before_eula_gate`
- [ ] Add parity test: `--report` output registry IDs and counts match official `server.jar --report` output for vanilla 26.1.2

## Migrated From Main Checklist: World Storage

- [x] Implement world folder layout.
- [x] Implement `level.dat`, `level.dat_old`, and session lock behavior.
- [x] Implement NBT binary format, compressed NBT, SNBT where needed, and visitor/traversal utilities.
- [x] Implement DataVersion tracking.
- [x] Implement DataFixer-equivalent world upgrade pipeline or explicit compatible upgrade tooling.
- [x] Implement region file format `.mca`.
- [x] Implement region compression types used by 26.1.2.
- [ ] Implement chunk serialization for blocks, biomes, heightmaps, block entities, entities, structures, ticks, lights, and post-processing.
- [x] Implement player data files.
- [x] Add a Mineflayer playerdata round-trip test that changes position, rotation, inventory, selected slot, health, food, XP, game mode, recipe book, and stats, disconnects, then reconnects in offline mode and verifies persistence.
- [x] Add raw 26.1.2 playerdata fallback coverage that sends movement, disconnects, restarts, and verifies the reconnect spawn position and rotation are loaded from compressed `playerdata/<uuid>.dat` while Mineflayer lacks target-protocol play support.
- [x] Add raw 26.1.2 selected-hotbar-slot persistence fallback coverage that sends `serverbound/minecraft:set_carried_item`, saves `SelectedItemSlot`, restarts, and verifies reconnect emits the saved `clientbound/minecraft:set_held_slot` while Mineflayer lacks target-protocol play support.
- [x] Add raw 26.1.2 selected-hotbar-slot bounds coverage that sends an out-of-range `serverbound/minecraft:set_carried_item` slot and verifies reconnect keeps vanilla-compatible slot `0` instead of persisting invalid `SelectedItemSlot`.
- [x] Add raw 26.1.2 playerdata login-state fallback coverage that seeds gzip playerdata with `Health`, `foodLevel`, `foodSaturationLevel`, `XpP`, `XpLevel`, `XpTotal`, and `SelectedItemSlot`, then verifies reconnect packets emit the saved health, food, XP, and held-slot values while Mineflayer lacks target-protocol play support.
- [x] Add raw 26.1.2 playerdata bounds fallback coverage that seeds out-of-range `Health`, `foodLevel`, `foodSaturationLevel`, `XpP`, `XpLevel`, `XpTotal`, and `SelectedItemSlot`, then verifies reconnect clamps login health, food, XP, and held-slot packets to vanilla-safe values.
- [x] Add raw 26.1.2 playerdata game-mode fallback coverage that seeds `playerGameType` and `previousPlayerGameType`, then verifies reconnect emits the saved game mode through play login, tab-list, and ability packets while out-of-range legacy IDs fall back to survival.
- [x] Add raw 26.1.2 server-properties game-mode fallback coverage that verifies fresh profiles use `gamemode`, saved profiles keep `playerGameType`, and `force-gamemode=true` overrides saved mode while preserving `previousPlayerGameType` in login packets.
- [x] Add raw 26.1.2 invalid `gamemode` property fallback coverage that verifies unknown game mode names behave like vanilla `GameType.byName(..., SURVIVAL)` and produce survival login, tab-list, and ability packets.
- [x] Add raw 26.1.2 numeric `gamemode` property coverage that verifies `gamemode=1` is parsed like vanilla `dispatchNumberOrString(GameType::byId, GameType::byName)` and emits creative login, tab-list, and ability packets.
- [x] Add raw 26.1.2 padded numeric `gamemode` property coverage that verifies `gamemode=01` follows vanilla `Integer.parseInt` handling and emits creative login, tab-list, and ability packets.
- [x] Add raw 26.1.2 out-of-range numeric `gamemode` property coverage that verifies `gamemode=99` follows vanilla `GameType.byId` out-of-bounds-to-zero behavior and emits survival login, tab-list, and ability packets.
- [x] Add raw 26.1.2 negative numeric `gamemode` property coverage that verifies `gamemode=-1` follows vanilla `GameType.byId` out-of-bounds-to-zero behavior and emits survival login, tab-list, and ability packets.
- [x] Add raw 26.1.2 spectator `gamemode` property fallback coverage that verifies fresh profiles emit spectator game mode with vanilla spectator ability flags before Mineflayer target-protocol support exists.
- [x] Add raw 26.1.2 effective game-mode save coverage that verifies completed logins write the effective `playerGameType` and nullable `previousPlayerGameType` back into gzip playerdata after default and forced game-mode resolution.
- [x] Add a Mineflayer offline-mode fresh-profile persistence test that logs in, disconnects without movement, and verifies which profile, stats, advancements, recipe, and playerdata files vanilla creates immediately versus at first save.
- [x] Add raw 26.1.2 fresh-profile file-creation fallback coverage that aborts after login success, verifies no early playerdata file exists, then completes play entry and disconnects to verify gzip-compressed playerdata is created while advancements/stats remain absent until progress exists.
- [x] Add a Mineflayer offline-mode reconnect-after-save test that records bot state before disconnect, waits for server-side save completion, reconnects, and verifies UUID-bound playerdata is loaded before the first visible spawn packet.
- [x] Add raw 26.1.2 reconnect-after-save fallback coverage that records position and held-slot state, waits for the UUID-named gzip playerdata file after clean disconnect, restarts, and verifies reconnect packets load the saved state before spawn while Mineflayer lacks target-protocol play support.
- [x] Add a Mineflayer offline-mode dirty-save test that disconnects immediately after movement, inventory, damage, and stat changes, then restarts and verifies vanilla-compatible flush timing and persisted data.
- [x] Add raw 26.1.2 dirty-save fallback coverage that sends movement plus selected-hotbar-slot changes, aborts immediately after first-tick actions, restarts, and verifies reconnect loads the saved position and held slot before spawn while Mineflayer lacks target-protocol play support.
- [x] Add a Mineflayer offline-mode abrupt-disconnect persistence test that destroys the client socket after inventory, position, and stat changes, then restarts and verifies vanilla-compatible last saved state and cleanup.
- [x] Add raw 26.1.2 abrupt-disconnect persistence fallback coverage that destroys the socket at the play boundary after dirty state changes, waits for gzip playerdata flush, restarts, and verifies the same offline UUID reloads the last saved state while Mineflayer lacks target-protocol play support.
- [x] Add a Mineflayer offline-mode first-login file creation test that verifies playerdata, stats, advancements, recipe book, and last-known-position files appear only at the vanilla-compatible save points for a newly generated profile.
- [x] Add raw 26.1.2 first-login file-creation fallback coverage that aborts after login success to verify no early playerdata, completes first play save to verify gzip playerdata, verifies recipe book is stored inside playerdata, and verifies no stats, advancements, recipe sidecar, or last-known-position sidecar files are created for a fresh profile while Mineflayer lacks target-protocol play support.
- [x] Add a Mineflayer offline-mode playerdata UUID ownership test that logs in two generated profiles, swaps or removes one playerdata file, reconnects both, and verifies vanilla-compatible recovery, reassignment refusal, or regeneration behavior.
- [x] Add raw 26.1.2 playerdata UUID ownership fallback coverage that logs in two offline profiles, verifies each UUID-named playerdata file reloads only its own position, verifies missing primary `.dat` falls back to `.dat_old`, removes both files, and verifies only that profile regenerates default state while the other UUID keeps its saved state.
- [x] Add a Mineflayer offline-mode playerdata-corruption login test that starts with truncated, wrong-compression, wrong-UUID, and wrong-dimension playerdata files, then verifies fallback spawn, warnings, and recovery match vanilla.
- [x] Add storage-level playerdata corruption coverage for vanilla `PlayerDataStorage` behavior: gzip playerdata saves rotate `<uuid>.dat_old`, corrupt `<uuid>.dat` is copied to `<uuid>_corrupted_<timestamp>.dat`, and load falls back to `.dat_old`.
- [x] Add raw 26.1.2 playerdata-corruption fallback coverage that creates primary and `.dat_old` playerdata through real joins, corrupts the primary `.dat`, restarts, verifies first spawn position is loaded from `.dat_old`, and confirms a `_corrupted_*.dat` copy is created while Mineflayer lacks target-protocol play support.
- [x] Implement advancements files.
- [x] Implement stats files.
- [x] Implement scoreboard save data.
- [x] Implement raids save data.
- [x] Implement map item save data.
- [x] Implement forced chunks save data.
- [x] Implement command storage.
- [x] Implement custom bossbar save data.
- [x] Implement random sequences save data.
- [x] Implement POI storage.
- [x] Implement entity region/storage behavior.
- [x] Implement durable write, temp-file, backup, and corruption handling semantics.
- [x] Implement symlink validation and path allow-list behavior.

## Migrated From Main Checklist: Source-Derived Granularity Appendix - Storage, NBT, And Datafix Coverage

- [x] Implement every NBT tag type: end, byte, short, int, long, float, double, byte array, string, list, compound, int array, and long array.
- [x] Implement NBT IO, size accounting, recursion/depth limits, streaming visitors, field selectors, SNBT parser, SNBT printer, text component visitor, and error reporting.
- [ ] Implement `LevelStorageSource`, `LevelStorageAccess`, `PrimaryLevelData`, `DerivedLevelData`, `ServerLevelData`, `WorldData`, `LevelSummary`, `LevelVersion`, and session locking.
- [ ] Implement `SavedDataStorage`, `PlayerDataStorage`, `CommandStorage`, tag value input/output helpers, and all level resource paths.
- [ ] Implement loot storage classes: loot tables, pools, parameters, contexts, validation context, built-in table IDs, container component manipulation, and validation reporting.
- [ ] Implement a datafix strategy covering schemas and fixes for blocks, block entities, entities, items, chunks, POIs, options, advancements, stats, scoreboards, structures, text components, villager data, worldgen settings, and versioned renames.
- [x] If a full DataFixerUpper-compatible pipeline is deferred, add explicit blockers preventing unsafe loading of worlds requiring unsupported migrations.
