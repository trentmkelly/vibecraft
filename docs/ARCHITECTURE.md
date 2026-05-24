# RustCraft Architecture

RustCraft targets Minecraft Java Edition 26.1.2 dedicated-server behavior.

Initial implementation choices:

- Language: Rust 2021.
- Runtime: standard library TCP/threading; no async runtime is currently part of the server architecture.
- Serialization: standard library for bootstrap files, `serde_json` where JSON helpers are useful, and project-owned NBT, packet, and JSON-facing codecs in RustCraft modules.
- Compression: `flate2` for zlib/gzip packet, log, NBT, and region-file compression; `lz4` for vanilla region-file compression option parity.
- Crypto: `aes` for the Minecraft AES/CFB8 stream used by login encryption and `sha1` for vanilla server-hash formatting.
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
- `gameplay`: combat, damage, status effects, weather, time, rules, block/item interactions, and scheduled world behavior.
- `command`: Brigadier-compatible command tree, selectors, command execution, functions.
- `ops`: bans, whitelist, ops, RCON, query, JSON-RPC management, status.
- `tests`: black-box parity harness against official `server.jar`.
