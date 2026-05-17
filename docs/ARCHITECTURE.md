# RustCraft Architecture

RustCraft targets Minecraft Java Edition 26.1.2 dedicated-server behavior.

Initial implementation choices:

- Language: Rust 2021.
- Runtime: standard library only until networking and async needs force a dependency.
- Serialization: standard library for bootstrap files; dedicated NBT, packet, and JSON codecs will be implemented in project modules.
- Compression: pending, expected zlib-compatible implementation for packet compression and region files.
- Crypto: pending, must support the vanilla login encryption flow.
- Persistence: project-owned world storage modules matching vanilla folder, NBT, region, entity, and POI formats.

Planned module boundaries:

- `cli`: vanilla-compatible dedicated server command line options.
- `eula`: `eula.txt` creation and agreement detection.
- `server_properties`: `server.properties` defaulting, parsing, preservation, and typed access.
- `network`: TCP transport, VarInt framing, packet codecs, compression, encryption, protocol states.
- `registry`: identifiers, resource keys, holders, tags, built-in and datapack registries.
- `resource`: vanilla resources, datapacks, reloads, tags, recipes, loot, advancements, worldgen data.
- `storage`: NBT, SNBT, datafix strategy, level data, region files, player data, saved data.
- `world`: dimensions, chunks, lighting, block states, fluids, ticks, worldgen.
- `entity`: entity base, living entities, players, AI, mobs, projectiles, vehicles.
- `command`: Brigadier-compatible command tree, selectors, command execution, functions.
- `ops`: bans, whitelist, ops, RCON, query, JSON-RPC management, status.
- `tests`: black-box parity harness against official `server.jar`.
