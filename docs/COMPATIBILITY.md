# RustCraft Compatibility Notes

Target: Minecraft Java Edition 26.1.2.

The Java decompilation in `../decompiled-server-26.1.2` is treated as the behavioral reference, but RustCraft must not copy Mojang source.

Known incomplete areas:

- Play-state gameplay is not implemented.
- Full vanilla datapack resources, dynamic registries, tags, recipes, loot, advancements, commands, chunks, entities, blocks, items, worldgen, lighting, and gameplay are not implemented.
- Online-mode authentication and secure chat are not implemented.
- DataFixer-compatible upgrades are not implemented.
- Packet compression and encryption are not fully implemented.
- JSON-RPC management server behavior is not implemented.

Intentional deviations:

- The dedicated server Swing GUI is intentionally unsupported for RustCraft's target platform. RustCraft runs as a headless dedicated server with console input, logs, RCON, query, and status interfaces; this does not affect vanilla client protocol compatibility.
- The chase remote-control debug server/client is intentionally unsupported. It is a developer/operator debug feature outside the vanilla client protocol and has no effect on unmodified client compatibility.
- The `use-native-transport` property is accepted and preserved for server.properties parity, but RustCraft uses Rust's standard `TcpListener`/`TcpStream` transport instead of Netty epoll/kqueue native transports. This is an implementation transport decision and does not change the vanilla TCP protocol surface exposed to clients.

Before any `CHECKLIST.md` item is checked, it must have corresponding code, docs, or test evidence in `RustCraft`.
