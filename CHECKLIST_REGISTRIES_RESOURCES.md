# Registries, Codecs, and Resources Checklist

Registry, codec, datapack, and resource-pack parity work moved out of the top-level checklist.

## Migrated From Main Checklist: Registries And Codecs

- [x] Implement root registry infrastructure matching `Registries` and `BuiltInRegistries`.
- [x] Implement resource keys, identifiers, holder references, holder sets, tags, and lifecycle metadata.
- [x] Implement frozen and mutable registry access phases.
- [x] Implement registry serialization/deserialization with data pack override behavior.
- [x] Implement bootstrapping for built-in registries before datapacks load.
- [x] Implement datapack-driven dynamic registries.
- [x] Implement registry sync during configuration state.
- [x] Implement exact registry IDs and element ordering expected by clients.
- [ ] Add a Mineflayer registry-sync test that captures configuration packets during offline-mode login and compares registry IDs, tag contents, known packs, and enabled feature order against official `server.jar`.
- [ ] Add a Mineflayer registry-login-diff test that runs the same offline-mode bot against RustCraft and official `server.jar`, then emits a compact registry/configuration diff whenever play-state entry fails.
- [ ] Add a Mineflayer offline-mode registry-size guard test that verifies large registry/tag payloads complete configuration without Mineflayer parser errors, truncated packets, or server-side compression regressions.
- [x] Implement codecs for JSON/NBT/network forms of registry-backed values.
- [x] Implement feature flag registry and enabled-feature negotiation.
- [x] Implement default enabled feature set for 26.1.2.
- [x] Implement tag loading, replacement, optional entries, and error reporting.
- [x] Implement reloadable server registries and resource reload dependency ordering.
- [ ] Add a Mineflayer datapack reload test that joins before and after `/reload`, verifies the bot survives registry/tag resync where vanilla does, and records any disconnect reason when vanilla kicks.
- [ ] Add a Mineflayer feature-flag/datapack mismatch test that attempts offline-mode login with changed enabled features or datapack registry contents and verifies vanilla-compatible configuration success or disconnect behavior.

## Migrated From Main Checklist: Resource Packs And Data Packs

- [x] Load vanilla built-in datapack from bundled resources.
- [x] Load world datapacks from `datapacks`.
- [x] Implement pack metadata parsing and compatibility checks.
- [x] Implement pack priority, enabling, disabling, safe mode, and reload.
- [x] Implement `data/minecraft/advancement`.
- [x] Implement `data/minecraft/banner_pattern`.
- [x] Implement cat, chicken, cow, frog, pig, wolf, and zombie nautilus variants.
- [x] Implement `data/minecraft/chat_type`.
- [x] Implement `data/minecraft/damage_type`.
- [x] Implement `data/minecraft/dialog`.
- [x] Implement `data/minecraft/dimension_type`.
- [x] Implement `data/minecraft/enchantment`.
- [x] Implement `data/minecraft/enchantment_provider`.
- [x] Implement `data/minecraft/instrument`.
- [x] Implement `data/minecraft/jukebox_song`.
- [x] Implement `data/minecraft/loot_table`.
- [x] Implement `data/minecraft/painting_variant`.
- [x] Implement `data/minecraft/recipe`.
- [x] Implement `data/minecraft/structure`.
- [x] Implement `data/minecraft/tags`.
- [x] Implement `data/minecraft/test_environment`.
- [x] Implement `data/minecraft/test_instance`.
- [x] Implement `data/minecraft/timeline`.
- [x] Implement `data/minecraft/trade_set`.
- [x] Implement `data/minecraft/trial_spawner`.
- [x] Implement `data/minecraft/trim_material`.
- [x] Implement `data/minecraft/trim_pattern`.
- [x] Implement `data/minecraft/villager_trade`.
- [x] Implement `data/minecraft/world_clock`.
- [x] Implement `data/minecraft/worldgen`.
- [x] Implement reload failure rollback and user-facing error reporting.
