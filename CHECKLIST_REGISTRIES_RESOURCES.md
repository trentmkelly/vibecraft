# Registries, Codecs, and Resources Checklist

Registry, codec, datapack, and resource-pack parity work moved out of the top-level checklist.

## Migrated From Main Checklist: Registries And Codecs

- [ ] Implement root registry infrastructure matching `Registries` and `BuiltInRegistries`.
- [ ] Implement resource keys, identifiers, holder references, holder sets, tags, and lifecycle metadata.
- [ ] Implement frozen and mutable registry access phases.
- [ ] Implement registry serialization/deserialization with data pack override behavior.
- [ ] Implement bootstrapping for built-in registries before datapacks load.
- [ ] Implement datapack-driven dynamic registries.
- [ ] Implement registry sync during configuration state.
- [ ] Implement exact registry IDs and element ordering expected by clients.
- [x] Add a Mineflayer registry-sync test that captures configuration packets during offline-mode login and compares registry IDs, tag contents, known packs, and enabled feature order against official `server.jar`. — `harness/mineflayer/registry_scenarios.mjs` `registry-sync` (captures-configuration-packets / compares-registry-ids / compares-tag-contents / compares-known-packs / compares-enabled-feature-order / official-server-oracle); `registry_scenarios.test.mjs` fail-closed (7 tests pass).
- [x] Add a Mineflayer registry-login-diff test that runs the same offline-mode bot against VibeCraft and official `server.jar`, then emits a compact registry/configuration diff whenever play-state entry fails. — `registry_scenarios.mjs` `registry-login-diff` (same-bot-against-vibecraft-and-official / compact-registry-diff-on-play-state-failure / configuration-diff-on-play-state-failure).
- [x] Add a Mineflayer offline-mode registry-size guard test that verifies large registry/tag payloads complete configuration without Mineflayer parser errors, truncated packets, or server-side compression regressions. — `registry_scenarios.mjs` `registry-size-guard` (large-registry-payload / large-tag-payload / configuration-completes / no-mineflayer-parser-errors / no-truncated-packets / no-compression-regression).
- [ ] Implement codecs for JSON/NBT/network forms of registry-backed values.
- [ ] Implement feature flag registry and enabled-feature negotiation.
- [ ] Implement default enabled feature set for 26.1.2.
- [ ] Implement tag loading, replacement, optional entries, and error reporting.
- [ ] Implement reloadable server registries and resource reload dependency ordering.
- [x] Add a Mineflayer datapack reload test that joins before and after `/reload`, verifies the bot survives registry/tag resync where vanilla does, and records any disconnect reason when vanilla kicks. — `harness/mineflayer/datapack_scenarios.mjs` (join-before-reload / run-reload-command / bot-survives-where-vanilla-survives / registry-tag-resync-observed / disconnect-reason-recorded-when-vanilla-kicks); `datapack_scenarios.test.mjs` passes (5 tests).
- [x] Add a Mineflayer feature-flag/datapack mismatch test that attempts offline-mode login with changed enabled features or datapack registry contents and verifies vanilla-compatible configuration success or disconnect behavior. — `datapack_scenarios.mjs` (changed-enabled-features / feature-flag-datapack-mismatch / vanilla-compatible-success-or-disconnect / disconnect-component-parity).

## Migrated From Main Checklist: Resource Packs And Data Packs

- [ ] Load vanilla built-in datapack from bundled resources.
- [ ] Load world datapacks from `datapacks`.
- [ ] Implement pack metadata parsing and compatibility checks.
- [ ] Implement pack priority, enabling, disabling, safe mode, and reload.
- [ ] Implement `data/minecraft/advancement`.
- [ ] Implement `data/minecraft/banner_pattern`.
- [ ] Implement cat, chicken, cow, frog, pig, wolf, and zombie nautilus variants.
- [ ] Implement `data/minecraft/chat_type`.
- [ ] Implement `data/minecraft/damage_type`.
- [ ] Implement `data/minecraft/dialog`.
- [ ] Implement `data/minecraft/dimension_type`.
- [ ] Implement `data/minecraft/enchantment`.
- [ ] Implement `data/minecraft/enchantment_provider`.
- [ ] Implement `data/minecraft/instrument`.
- [ ] Implement `data/minecraft/jukebox_song`.
- [ ] Implement `data/minecraft/loot_table`.
- [ ] Implement `data/minecraft/painting_variant`.
- [ ] Implement `data/minecraft/recipe`.
- [ ] Implement `data/minecraft/structure`.
- [ ] Implement `data/minecraft/tags`.
- [ ] Implement `data/minecraft/test_environment`.
- [ ] Implement `data/minecraft/test_instance`.
- [ ] Implement `data/minecraft/timeline`.
- [ ] Implement `data/minecraft/trade_set`.
- [ ] Implement `data/minecraft/trial_spawner`.
- [ ] Implement `data/minecraft/trim_material`.
- [ ] Implement `data/minecraft/trim_pattern`.
- [ ] Implement `data/minecraft/villager_trade`.
- [ ] Implement `data/minecraft/world_clock`.
- [ ] Implement `data/minecraft/worldgen`.
- [ ] Implement reload failure rollback and user-facing error reporting.
