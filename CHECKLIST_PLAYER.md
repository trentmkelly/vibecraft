# Player Entity and State Checklist

## Relevant Java Source Files

- `decompiled-server-26.1.2/net/minecraft/world/entity/player/Player.java` — base player logic (shared client/server)
- `decompiled-server-26.1.2/net/minecraft/server/level/ServerPlayer.java` — server-side player
- `decompiled-server-26.1.2/net/minecraft/server/players/PlayerList.java` — player join/leave management
- `decompiled-server-26.1.2/net/minecraft/network/ServerGamePacketListenerImpl.java` — play state packet handler
- `decompiled-server-26.1.2/net/minecraft/world/entity/player/Inventory.java` — player inventory
- `decompiled-server-26.1.2/net/minecraft/world/entity/player/Abilities.java` — player ability flags
- `decompiled-server-26.1.2/net/minecraft/world/food/FoodData.java` — hunger/saturation/exhaustion
- `decompiled-server-26.1.2/net/minecraft/world/entity/player/Player.java` — XP/level
- `decompiled-server-26.1.2/net/minecraft/world/level/GameType.java` — game mode enum
- `decompiled-server-26.1.2/net/minecraft/server/level/ServerPlayerGameMode.java` — server-side game mode logic
- `decompiled-server-26.1.2/net/minecraft/world/phys/AABB.java` — bounding box (collision, reach)
- `decompiled-server-26.1.2/net/minecraft/world/entity/Relative.java` — relative flag enum for teleport
- `decompiled-server-26.1.2/net/minecraft/world/level/storage/PlayerDataStorage.java` — playerdata files
- `RustCraft/src/player.rs` — RustCraft player
- `RustCraft/src/player_entity.rs` — RustCraft player entity
- `RustCraft/src/player_inventory.rs` — RustCraft player inventory
- `RustCraft/src/movement_validation.rs` — RustCraft movement validation
- `RustCraft/src/movement_physics.rs` — RustCraft movement physics
- `RustCraft/src/respawn.rs` — RustCraft respawn logic

## Player State Synchronization Tests

- [ ] Add Mineflayer player-state test: after offline-mode join verify health (20.0), food (20), saturation (5.0 default), XP (0), game mode (matches server default), permissions (non-op), recipe book state (empty)
- [ ] Add Mineflayer offline-mode game-mode-persistence test: change bot between survival/creative/adventure/spectator, reconnect each time; verify `force-gamemode` overrides and saved game-mode match vanilla
- [ ] Add Mineflayer offline-mode respawn-after-relogin test: kill bot, disconnect on death screen, reconnect; verify vanilla-compatible death/respawn state recovery (no extra death, correct spawn point)
- [ ] Add Mineflayer death/respawn test: kill bot in offline mode, verify death message packet, respawn packet flow (`ClientboundRespawnPacket`), inventory/XP rules on death (keepInventory gamerule), respawn spawn position, post-respawn ability flags

## Player Entity Parity

- [ ] Implement player reach distance: survival = 5.0 blocks (server-authoritative), creative = 5.0 (place) / 5.0 (interact), attack = 3.0; verify server-side reach check matches vanilla
- [ ] Implement hunger/saturation/exhaustion tick: sprint (+0.1/tick), jump (+0.05), attack (+0.3 per hit), damage-absorption calculation; starvation damage at 0 food on hard mode
- [ ] Implement food level effects: fast healing above 18 food (0.5 HP/4 ticks in Java), normal healing above 0, speed reduction below 6 (5 for walking)
- [ ] Implement exhaustion-to-saturation-to-food-level drain: exhaustion ≥ 4.0 drains 1 saturation; 0 saturation drains 1 food
- [ ] Implement XP orb pickup and merge: orbs within 1.5 blocks merge toward player, `addExperience()` with level-up threshold table
- [ ] Implement XP level→point threshold: `getXpNeededForNextLevel()` = (level ≥ 30) ? 112 + (level-30)*9 : (level ≥ 15) ? 37 + (level-15)*5 : 7 + level*2
- [ ] Implement player death drops: drop all inventory on death in non-keepInventory mode; keep bound items if `CurseOfBinding`; XP orb generation proportional to XP level
- [ ] Implement player respawn: consume `respawnPosition` if set and block still valid, else find default world spawn; apply respawn invulnerability ticks
- [ ] Implement `Abilities` flags: invulnerable, flying, can-fly, instant-build (creative), flying-speed (0.05 walk / 0.1 fly), walking-speed (0.1); sync on game mode change
- [ ] Implement `ServerPlayerGameMode`: survival break-speed calculation per tool/block, creative instant-break, spectator no-interaction, adventure restriction (no break unless CanDestroy tag)
- [ ] Implement spawn protection check: blocks within `spawn-protection` radius from world spawn cannot be broken by non-ops
- [ ] Implement player abilities packet sync on: game mode change, op status change, allow-flight property change

## Movement and Physics

- [ ] Add Mineflayer movement tests: walking, jumping, sneaking, sprinting, falling, invalid-movement correction, and chunk-boundary crossing in offline mode
- [ ] Add Mineflayer invalid-movement tests: out-of-bounds, too-fast, illegal stance, no-clip, and flight-like movement; verify server correction or kick matches vanilla
- [ ] Add Mineflayer teleport/position-confirm tests: server-issued teleports, relative-flag movement, yaw/pitch corrections, cross-chunk teleports, dimension changes, stale teleport confirmations
- [ ] Implement `ServerboundMovePlayerPacket` validation: moved-too-quickly check (`abs(dx²+dy²+dz²) > 100²` → kick), invalid-position check (NaN/Inf → kick), flying check (no-fly mode + airborne > 80 ticks → kick if allow-flight=false)
- [ ] Implement step height: 0.6 blocks default, auto-step up single block
- [ ] Implement vehicle movement validation: `ServerboundMoveVehiclePacket` with vehicle-specific speed limits
- [ ] Implement `RelativeMovement` flag encoding for `ClientboundPlayerPositionPacket`: bitmask using `Relative` enum with correct flag values (X=0x01, Y=0x02, Z=0x04, Y_ROT=0x08, X_ROT=0x10, DELTA_X=0x20, DELTA_Y=0x40, DELTA_Z=0x80, ROTATE_DELTA=0x100)

## Inventory Tests

- [ ] Add Mineflayer equipment and inventory sync tests: armor, offhand, selected hotbar slot, item pickup, item drop, respawn retention rules, disconnect/reconnect persistence
- [ ] Add Mineflayer offline-mode inventory-login-persistence test: give items to bot, disconnect before and after explicit save, reconnect; verify vanilla-compatible inventory restore timing
- [ ] Add Mineflayer inventory correction tests: desync slot clicks, carried items, selected hotbar slot, creative actions, drop packets; verify vanilla-compatible corrections
- [ ] Add Mineflayer offline-mode login-inventory baseline test: fresh generated profile receives empty 46-slot inventory, selected slot 0, empty cursor, empty recipe book before any scripted action
- [ ] Add Mineflayer offline-mode window lifecycle tests: open, click, close, reopen, disconnect mid-window for player inventory, chest, furnace, crafting table, anvil, and merchant menus

## Multiplayer Visibility Tests

- [ ] Add Mineflayer offline-mode login visibility test: bot is added to tab list, spawned for nearby bots, visible to command selectors only after vanilla play-state boundary
- [ ] Add Mineflayer multi-bot visibility test: join two bots; verify tab-list entries, spawn/despawn packets, relative movement, sneaking/sprinting flags, held items, disconnect cleanup
- [ ] Add Mineflayer offline-mode entity-tracking distance test: move bots across tracking thresholds; verify spawn/metadata/velocity/equipment/remove packets match vanilla timing

## Player Data Persistence Tests

- [ ] Add Mineflayer playerdata round-trip test: change position, rotation, inventory, selected slot, health, food, XP, game mode, recipe book, and stats; disconnect; reconnect; verify all fields restored
- [ ] Add Mineflayer offline-mode fresh-profile persistence test: log in, disconnect without movement; verify which profile/stats/advancements/recipe/playerdata files vanilla creates immediately vs. at first save
- [ ] Add Mineflayer offline-mode reconnect-after-save test: record bot state before disconnect, wait for server-side save, reconnect; verify playerdata is loaded before first visible spawn packet
- [ ] Add Mineflayer offline-mode dirty-save test: disconnect immediately after movement/inventory/damage/stat changes; restart; verify vanilla-compatible flush timing and persisted data
- [ ] Add Mineflayer offline-mode abrupt-disconnect persistence test: destroy client socket after inventory/position/stat changes; restart; verify vanilla-compatible last-saved state and cleanup
- [ ] Add Mineflayer offline-mode first-login file creation test: verify playerdata/stats/advancements/recipe-book/last-known-position files appear only at vanilla-compatible save points
- [ ] Add Mineflayer offline-mode playerdata UUID ownership test: log in two generated profiles, swap/remove one playerdata file, reconnect both; verify vanilla-compatible recovery behavior
- [ ] Add Mineflayer offline-mode playerdata-corruption login test: start with truncated/wrong-compression/wrong-UUID/wrong-dimension playerdata; verify fallback spawn, warnings, and recovery match vanilla

## Chunk Streaming Tests

- [ ] Add Mineflayer chunk-streaming test: join offline mode, wait for initial chunks, change view-distance and simulation-distance, move across chunk boundaries; verify chunk load/unload events match vanilla ordering
- [ ] Add Mineflayer offline-mode slow-initial-chunk test: delay first chunk availability; verify bot remains connected through vanilla-compatible keepalives, loading state, and eventual spawn readiness
- [ ] Add Mineflayer chunk-resend regression test: reconnect bot in same chunk, teleport across dimensions or long distances; verify stale chunks are unloaded before new terrain is accepted
- [ ] Add Mineflayer forced-chunk visibility test: use `/forceload`, move bot away and back; verify chunk data remains available consistently with vanilla
- [ ] Add Mineflayer first-login spawn parity test: compare initial spawn block, yaw/pitch, dimension, world seed, and spawn protection behavior against official `server.jar`
- [ ] Add Mineflayer spawn-area safety test: repeatedly create seeded offline-mode worlds, join bot; verify vanilla-compatible spawn search, collision-free placement, immediate chunk availability

## Post-Login Readiness Tests

- [ ] Add Mineflayer offline-mode post-login readiness test: wait for first physics tick; verify movement, chat, command suggestions, inventory window ID, and chunk visibility all usable without retry sleeps
- [ ] Add Mineflayer login-to-play timeline test: record bot events from TCP connect through first physics tick; compare ordering, packet gaps, timeout thresholds against official `server.jar`
- [ ] Add Mineflayer offline-mode first-tick test: verify bot can send movement, chat, command, inventory, block-look packets immediately after play state without race-condition disconnects
- [ ] Add Mineflayer offline-mode first-action matrix: send movement, chat, command, inventory click, block dig, and block place immediately after spawn; verify vanilla-compatible success or correction
- [ ] Add Mineflayer offline-mode reconnect-at-play-boundary test: disconnect at join-game, at first chunk, at first physics tick; verify player cleanup and next-login parity
- [ ] Add Mineflayer offline-mode play-readiness race test: repeat login-to-first-action under randomized chunk delays; fail when any action succeeds only after an arbitrary sleep

## Gamerule / Advancement / Stats Tests

- [ ] Add Mineflayer gamerule tests: toggle `keepInventory`, `doImmediateRespawn`, `sendCommandFeedback`, `doDaylightCycle`, `mobGriefing`; verify client-observable behavior
- [ ] Add Mineflayer stats/advancement tests: perform movement, mining, crafting, death, and recipe unlock actions; verify client updates and saved JSON files after reconnect
- [ ] Add Mineflayer offline-mode scoreboard objective lifecycle tests: create, update, display, hide, persist, remove objectives while bots online and after reconnect
- [ ] Add Mineflayer scoreboard/team tests: sidebar/list/below-name displays, team color/prefix/suffix, nametag visibility, collision rules, reconnect persistence

## Multiplayer Management Tests

- [ ] Add Mineflayer multiplayer lifecycle tests: join broadcast, tab-list add/remove, quit message, kick reason, respawn, transfer rejection/acceptance, reconnect persistence
- [ ] Add Mineflayer offline-mode login-queue ordering test: release bots from handshake/configuration gates in controlled order; verify accepted player order, tab-list order, join messages match vanilla
- [ ] Add Mineflayer simultaneous-offline-login tests: multiple profiles, join order, spawn collision, chat visibility, player list latency, disconnect cleanup
- [ ] Add Mineflayer offline-mode join/quit broadcast tests: first-login, returning-login, duplicate-login, kick, timeout, crash-disconnect messages vs. official `server.jar`
- [ ] Add Mineflayer offline-mode same-tick login/logout tests: connect/disconnect bots in rapid succession; verify tab-list/entity IDs/keepalives/playerdata writes don't leak
- [ ] Add Mineflayer offline-mode duplicate-session cleanup test: reconnect before previous TCP closes; verify entity removal, tab-list replacement, playerdata ownership, kicked-message parity
- [ ] Add Mineflayer offline-mode rapid reconnect tests: repeatedly connect/disconnect same bot name; verify stale entities, tab-list rows, keepalive tasks, player files cleaned up
- [ ] Add Mineflayer offline-mode mixed-profile multiplayer tests: op/non-op/whitelisted/banned/duplicate profiles in one run; verify accepted ordering, rejection reasons, broadcasts, tab-list state
- [ ] Add Mineflayer offline-mode reconnect-after-kick test: kick, ban, unban, reconnect; verify stale session cleanup and vanilla-compatible kicked messages

## Presentation Tests

- [ ] Add Mineflayer presentation tests: action bar, title/subtitle/times, bossbar add/update/remove, tab-list header/footer, death message formatting
- [ ] Add Mineflayer tab-list mutation tests: change latency, display name, game mode, listed flag, hat visibility, list order while bots online; compare packet/event order against vanilla
- [ ] Add Mineflayer offline-mode profile-property tab-list test: empty and synthetic profile properties; verify vanilla-compatible tab-list serialization
