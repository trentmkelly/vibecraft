# Minecraft Server Rebuild Checklist

Goal: rebuild a Minecraft Java Edition 26.1.2 compatible server in another programming language, with behavior and wire compatibility matching the official dedicated server.

This top-level file is now an index. Mark a subsystem row only after every item in the linked subsystem checklist is complete.

## Subsystem Checklists

- [x] Complete everything in [CHECKLIST_PROJECT_FOUNDATION.md](CHECKLIST_PROJECT_FOUNDATION.md) for project foundation and compatibility ground rules, then mark this off.
- [x] Complete everything in [CHECKLIST_SERVER_RUNTIME.md](CHECKLIST_SERVER_RUNTIME.md) for server bootstrap, configuration, runtime, and operations, then mark this off.
- [x] Complete everything in [CHECKLIST_REGISTRIES_RESOURCES.md](CHECKLIST_REGISTRIES_RESOURCES.md) for registries, codecs, datapacks, and resource packs, then mark this off.
- [ ] Complete everything in [CHECKLIST_NETWORK_GAME.md](CHECKLIST_NETWORK_GAME.md) for network transport and protocol packet parity, then mark this off.
- [x] Complete everything in [CHECKLIST_AUTH_CHAT.md](CHECKLIST_AUTH_CHAT.md) for authentication, secure profiles, and chat trust, then mark this off.
- [ ] Complete everything in [CHECKLIST_STORAGE.md](CHECKLIST_STORAGE.md) for world storage, NBT, datafixing, and persisted files, then mark this off.
- [ ] Complete everything in [CHECKLIST_WORLDGEN.md](CHECKLIST_WORLDGEN.md) for dimensions, chunk generation, chunk streaming, and vanilla worldgen parity, then mark this off.
- [x] Complete everything in [CHECKLIST_BLOCKS.md](CHECKLIST_BLOCKS.md) for blocks, block states, block behavior, and block-entity integration, then mark this off.
- [x] Complete everything in [CHECKLIST_BLOCK_ENTITIES.md](CHECKLIST_BLOCK_ENTITIES.md) for block entity implementations, then mark this off.
- [x] Complete everything in [CHECKLIST_ITEMS.md](CHECKLIST_ITEMS.md) for items, inventories, item stacks, and crafting integration, then mark this off.
- [x] Complete everything in [CHECKLIST_CONTAINERS.md](CHECKLIST_CONTAINERS.md) for container menus and inventory transaction behavior, then mark this off.
- [x] Complete everything in [CHECKLIST_RECIPES.md](CHECKLIST_RECIPES.md) for recipe loading and execution, then mark this off.
- [ ] Complete everything in [CHECKLIST_MOBS.md](CHECKLIST_MOBS.md) for entities, mobs, AI, bosses, raids, and vehicles, then mark this off.
- [x] Complete everything in [CHECKLIST_PLAYER.md](CHECKLIST_PLAYER.md) for player state, movement, multiplayer operations, and presentation data, then mark this off.
- [ ] Complete everything in [CHECKLIST_GAMEPLAY.md](CHECKLIST_GAMEPLAY.md) for gameplay mechanics, combat, effects, weather, time, and world border, then mark this off.
- [x] Complete everything in [CHECKLIST_COMMANDS.md](CHECKLIST_COMMANDS.md) for commands, functions, command blocks, and command tests, then mark this off.
- [x] Complete everything in [CHECKLIST_GAME_STATE.md](CHECKLIST_GAME_STATE.md) for game rules, scoreboards, teams, stats, and advancements, then mark this off.
- [ ] Complete everything in [CHECKLIST_LOOT.md](CHECKLIST_LOOT.md) for loot, trading, economy, and rewards, then mark this off.
- [x] Complete everything in [CHECKLIST_HARNESS.md](CHECKLIST_HARNESS.md) for vanilla-client, Mineflayer, parity, fuzz, performance, and soak harnesses, then mark this off.
- [ ] Complete everything in [CHECKLIST_RELEASE.md](CHECKLIST_RELEASE.md) for release and completion gates, then mark this off.

## Milestone Plan

- [x] Milestone 1: status ping server with correct MOTD/version/player sample response.
- [x] Milestone 2: offline-mode login through configuration into a void world.
- [ ] Milestone 3: chunk serialization and static flat terrain visible to vanilla client.
- [ ] Milestone 4: player movement, chat, keepalive, disconnect, and save/load.
- [ ] Milestone 5: block placement/breaking, inventory, item stacks, and basic survival loop.
- [ ] Milestone 6: commands, permissions, ops, whitelist, bans, RCON, query, and console operations.
- [ ] Milestone 7: region storage, real chunk loading/saving, lighting, and scheduled ticks.
- [ ] Milestone 8: vanilla datapacks, registries, recipes, loot, tags, and reload.
- [ ] Milestone 9: deterministic overworld/nether/end generation.
- [ ] Milestone 10: full entity set, AI, combat, spawning, bosses, raids, and trading.
- [ ] Milestone 11: complete command parity and command test coverage.
- [ ] Milestone 12: online-mode authentication, secure profiles, signed chat, and reporting-related metadata.
- [ ] Milestone 13: management server, dialogs, transfers, waypoints, timelines, and version-specific 26.1.2 features.
- [ ] Milestone 14: broad vanilla parity validation using automated black-box comparisons.
- [ ] Milestone 15: production hardening, monitoring, backups, crash recovery, and operator documentation.
