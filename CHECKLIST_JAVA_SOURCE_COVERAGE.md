# Java Source Coverage Checklist

This checklist tracks source families from `decompiled-server-26.1.2/net/minecraft` that are not completely covered by the subsystem checklists. The Java tree is the authoritative behavioral source; each item needs design notes, Rust implementation tasks, and parity tests or an explicit compatibility decision before it can be considered complete.

## Source Tree Inventory

- [ ] Maintain a generated inventory of all 4,779 Java source files and reconcile it against this checklist whenever the decompiled source changes.
- [ ] Keep [CHECKLIST_JAVA_CLASSES.md](CHECKLIST_JAVA_CLASSES.md) synchronized with the decompiled Java tree so every Java source file has an explicit per-class audit/port/parity-test task.
- [ ] Keep [CHECKLIST_RUST_SOURCE_AUDIT.md](CHECKLIST_RUST_SOURCE_AUDIT.md) synchronized with VibeCraft implementation and harness sources so Rust-only work cannot bypass Java parity/checklist review.
- [ ] Keep [CHECKLIST_VANILLA_DATA_RESOURCES.md](CHECKLIST_VANILLA_DATA_RESOURCES.md) synchronized with bundled vanilla data/assets so data-driven behavior is tracked at resource-file granularity.
- [ ] Add a class-by-class coverage manifest that maps each Java file to one Rust module, one checklist item, and one parity-test strategy.
- [ ] Add a CI/lint check that fails when a Java source file is not represented by the coverage manifest.
- [ ] Add an explicit deferral mechanism for client-only, GUI-only, launcher-only, or unsupported management features so omissions are intentional and searchable.

## Top-Level Runtime Utilities

- [ ] Audit and port or explicitly defer top-level classes: `SharedConstants`, `DetectedVersion`, `WorldVersion`, `CrashReport`, `CrashReportCategory`, `CrashReportDetail`, `ReportedException`, `ReportType`, `SystemReport`, uncaught exception handlers, `Optionull`, `IdentifierException`, `CharPredicate`, `ChatFormatting`, `TracingExecutor`, and package annotations.
- [ ] Add parity tests for crash report formatting, system report sections, version metadata, shared constants, chat formatting codes, and identifier/character validation errors.

## Core, Resources, Tags, And Registries

- [ ] Complete class-by-class coverage for `core` (38 files), `core/component` (30), `core/particles` (19), `core/dispenser` (12), `core/cauldron` (3), and `core/registries` (4).
- [ ] Complete class-by-class coverage for `resources` (15), including registry codecs, registry load tasks, dependent resource naming, file-to-id conversion, registry validators, and network/resource-manager load tasks.
- [ ] Complete class-by-class coverage for `tags` (27), including every vanilla tag key family, tag builders/loaders/files/entries, and network tag serialization.
- [ ] Add parity tests that compare registry IDs, holder behavior, tag membership, component patches, particle option codecs, cauldron interactions, and dispenser behavior against Java fixtures.

## Commands

- [ ] Complete class-by-class coverage for `commands` (14), `commands/arguments` (69), `commands/execution` (18), `commands/functions` (7), and `commands/synchronization` (12), including parsing, suggestions, execution task scheduling, function loading, and Brigadier synchronization.
- [ ] Add command-tree and parser parity tests for every Java argument type, including block/item predicates, coordinates, selectors, NBT paths, ranges, resource keys, styles, and time/objective/team parsers.

## Network And Serialization

- [ ] Complete class-by-class coverage for `network` (42), `network/chat` (63), `network/codec` (7), `network/protocol` (293), and `network/syncher` (6).
- [ ] Add generated packet manifests for all handshake, status, login, configuration, common, cookie, ping, and game packet classes, including direction, state, packet ID, fields, codecs, and malformed-input behavior.
- [ ] Add parity tests for stream codecs, registry-friendly byte buffers, chat types, last-seen handling, message signatures, synched entity data serializers, cookie packets, transfer packets, and common play/configuration shared packets.

## Data, Packs, And Generated Reports

- [ ] Complete class-by-class coverage for `data` (10), `data/advancements` (10), `data/info` (8), `data/loot` (22), `data/metadata` (2), `data/recipes` (17), `data/registries` (5), `data/structures` (5), `data/tags` (30), and `data/worldgen` (56).
- [ ] Add tasks and tests for every data generator provider, including generated reports for blocks, items, commands, registries, packs, structures, tags, recipes, loot, advancements, and worldgen bootstrap data.
- [ ] Compare generated VibeCraft reports against official `server.jar --reports` output for Minecraft 26.1.2.

## Server Runtime And Operations

- [ ] Complete class-by-class coverage for `server` (27), `server/dedicated` (7), `server/level` (42), `server/network` (28), `server/players` (19), `server/permissions` (12), `server/rcon` (9), `server/bossevents` (3), `server/chase` (3), `server/advancements` (2), `server/notifications` (5), `server/waypoints` (2), and `server/gui` (4, likely deferred for headless VibeCraft).
- [ ] Complete class-by-class coverage for `server/jsonrpc` (64), including management API schemas, internal methods, security, websocket behavior, data providers, notification fanout, request lifecycle, and shutdown semantics.
- [ ] Complete class-by-class coverage for `server/packs` (55), including repositories, linkfs, metadata, resource-pack discovery, selection, validation, reload behavior, and pack failure handling.
- [ ] Add integration tests for dedicated server options, RCON/query, player list management, bans/ops/whitelist, permission levels, boss events, notifications, JSON-RPC management, server packs, and waypoint updates.

## Storage, NBT, DataFix, And FileFix

- [ ] Complete class-by-class coverage for `nbt` (36), `nbt/visitors` (7), `util/datafix` (392), `util/filefix` (56), and `util/worldupdate` (6).
- [ ] Decide whether to implement a DataFixerUpper-compatible pipeline or strictly block legacy world versions; document the supported DataVersion range for every persisted file type.
- [ ] Add parity tests for NBT text/binary/SNBT IO, visitors, size trackers, NBT paths, datafix schemas/fixes, file-fix virtual filesystem operations, chunk/entity/POI upgrade passes, and `--forceUpgrade` behavior.

## Util Libraries

- [ ] Complete class-by-class coverage for `util` (92), `util/context` (4), `util/debug` (19), `util/debugchart` (8), `util/eventlog` (4), `util/monitoring` (2), `util/parsing` (29), `util/profiling` (70), `util/random` (4), `util/thread` (9), and `util/valueproviders` (18).
- [ ] Add parity tests for random sources, weighted/random selection, value providers, thread/task helpers, profiling/JFR/debug charts, event logs, CSV/log utilities, time utilities, parsing combinators, and context maps.

## Gameplay Data Systems

- [ ] Complete class-by-class coverage for `advancements` (16), `advancements/criterion` (83), `recipebook` (3), `stats` (10), `sounds` (6), `locale` (3), `references` (3), `world/clock` (10), `world/timeline` (5), and `world/waypoints` (9).
- [ ] Add parity tests for every advancement criterion trigger/predicate, recipe-book serialization/update rules, statistics counters, sound event registries, localization fallback, generated ID references, world clocks, timelines, and waypoint packet/state behavior.

## World And Gameplay Mechanics

- [ ] Complete class-by-class coverage for `world` (25), `world/attribute` (28), `world/damagesource` (12), `world/effect` (20), `world/flag` (7), `world/food` (5), `world/inventory` (64), `world/item` (313), `world/phys` (27), `world/scores` (15), and `world/ticks` (14).
- [ ] Add parity tests for attributes/modifiers, damage sources/scaling, effects and hidden-effect chains, feature flags, food data, menus/inventories, every item class, physical shapes/raycasting, scoreboards/teams/objectives, and scheduled tick containers.

## Entity And AI Source Families

- [ ] Complete class-by-class coverage for `world/entity` (708), including base entity state, entity types, synched data, save/load, interaction, projectiles, vehicles, decorations, item entities, players, AI, navigation, behaviors, sensors, memory, schedules, raids, variants, bosses, monsters, animals, NPCs, ambient mobs, and entity events.
- [ ] Add parity tests for every entity family, every AI goal/behavior source family, path navigation, sensors/memory modules, spawn rules, despawn rules, loot/drop hooks, metadata, save NBT, damage/combat hooks, and client packet surfaces.

## World Level, Worldgen, And Structures

- [ ] Complete class-by-class coverage for `world/level` (1,297), including blocks, block states, block entities, materials, chunks, chunk storage/status, dimensions, lighting, redstone, border, game events/vibrations, pathfinding, portals, saved data, validation, entity storage, biomes, and all levelgen families.
- [ ] Add parity tests for every block class, block state property, block entity type, level/chunk lifecycle, dimension rules, lighting engine, redstone simulation, game events/vibrations, pathfinding, portals, saved data, and validation path.
- [ ] Add source-family tasks for all levelgen subpackages: noise, density functions, surface rules, carvers, features, placements, configured features, structures, jigsaw pools, templates, processors, flat/debug/noise generators, spawn placement, blending, retrogen, height providers, and random state.

## GameTest And Debug Tooling

- [ ] Complete class-by-class coverage for `gametest` (2), `gametest/framework` (43), and `gizmos` (15).
- [ ] Decide which Java GameTest server APIs VibeCraft will support directly versus through external parity harnesses, then add tests for structure templates, assertions, batches, listeners, reports, and debug marker/gizmo output.

## Generated Package Coverage Index

Generated from `decompiled-server-26.1.2/net/minecraft` on 2026-05-24. Each package bucket must be expanded into class-level tasks during implementation planning.

- [ ] `net/minecraft/CharPredicate.java` (1 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/ChatFormatting.java` (1 Java files): add class-level Rust port tasks and parity tests.
- [x] `net/minecraft/CrashReport.java` (1 Java files): class-level Rust port task completed by `src/crash.rs` and `src/crash/tests.rs`, including details/friendly report formatting, save-once behavior, stack-trace category tracking, throwable unwrapping, and preload tests.
- [ ] `net/minecraft/CrashReportCategory.java` (1 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/CrashReportDetail.java` (1 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/DefaultUncaughtExceptionHandler.java` (1 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/DefaultUncaughtExceptionHandlerWithName.java` (1 Java files): add class-level Rust port tasks and parity tests.
- [x] `net/minecraft/DetectedVersion.java` (1 Java files): class-level Rust port task completed by `src/world_version.rs`, including built-in fallback and real `version.json` parsing tests.
- [ ] `net/minecraft/IdentifierException.java` (1 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/Optionull.java` (1 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/ReportType.java` (1 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/ReportedException.java` (1 Java files): add class-level Rust port tasks and parity tests.
- [x] `net/minecraft/SharedConstants.java` (1 Java files): class-level Rust port task completed by `src/shared_constants.rs`, including constants, debug-property gates, version state, protocol, debug terrain, and static initializer tests.
- [ ] `net/minecraft/SuppressForbidden.java` (1 Java files): add class-level Rust port tasks and parity tests.
- [x] `net/minecraft/SystemReport.java` (1 Java files): class-level Rust port task completed by `src/system_report.rs`, including ordered entries, memory/JVM flag formatting, hardware/storage collection, error fallback, and output formatting tests.
- [ ] `net/minecraft/TracingExecutor.java` (1 Java files): add class-level Rust port tasks and parity tests.
- [x] `net/minecraft/WorldVersion.java` (1 Java files): class-level Rust port task completed by `src/world_version.rs`, including `WorldVersion.Simple` field access, pack-type switching, and `DataVersion` compatibility tests.
- [ ] `net/minecraft/advancements` (16 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/advancements/criterion` (83 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/commands` (14 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/commands/arguments` (41 Java files): add class-level Rust port tasks and parity tests.
- [x] `net/minecraft/commands/arguments/blocks` (5 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: `BlockInput`, `BlockPredicateArgument`, `BlockStateArgument`, and `BlockStateParser` are modeled in `command_block_arguments` with Java-backed block-state/property/NBT matching, placement/update, block and tag predicate parsing, vague-property validation, parser errors/cursor resets, serialization, suggestion, and package-info tests. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_arguments`.
- [x] `net/minecraft/commands/arguments/coordinates` (11 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: block/column position arguments are modeled in `command_block_position_arguments`, world/local/rotation/vector coordinates are modeled in `command_coordinate_arguments`, and swizzle parsing is modeled in `command_swizzle_argument`, with Java-backed factory, example, parse, cursor-reset, suggestion, validation, coordinate-resolution, axis, and package-info tests. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_position_arguments`, `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 coordinate_arguments`, and `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 swizzle_argument`.
- [x] `net/minecraft/commands/arguments/item` (7 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: `ComponentPredicateParser`, `FunctionArgument`, `ItemArgument`, `ItemInput`, `ItemParser`, and `ItemPredicateArgument` are modeled in `command_item_arguments` with Java-backed item/component parser, predicate grammar, stack construction, function/tag resolution, suggestion, error, cursor-reset, and package-info tests. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 item_arguments`.
- [ ] `net/minecraft/commands/arguments/selector` (3 Java files): add class-level Rust port tasks and parity tests.
- [x] `net/minecraft/commands/arguments/selector/options` (2 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: `EntitySelectorOptions` is modeled in `command_selector` with Java-backed registered option, inversion, validation/error, duplicate/inapplicable-state, sort, player-only narrowing, score/NBT/predicate, and advancement criterion tests, and package-info is audited as the nullability marker. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 command_selector`.
- [x] `net/minecraft/commands/execution` (11 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: chain modifiers, queue entries, custom executors/modifiers, entry actions, execution context/control, frames, trace callbacks, unbound actions, and package-info are modeled in `command_execution` with Java-backed queueing, callback, tracer, limit, fork/return, bind, and control-flow tests. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 command_execution`.
- [x] `net/minecraft/commands/execution/tasks` (7 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: build-context scheduling, function calls, continuations, command execution, fallthrough, isolated calls, and package-info are modeled in `command_execution` with Java-backed fork/return, cost, trace, frame, queueing, task-provider, and discard behavior tests. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 command_execution`.
- [x] `net/minecraft/commands/functions` (7 Java files): class-level Rust port tasks and parity tests are tracked in `CHECKLIST_JAVA_CLASSES.md` and verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 command_function`.
- [x] `net/minecraft/commands/synchronization` (6 Java files): class-level Rust port tasks and parity tests are tracked in `CHECKLIST_JAVA_CLASSES.md` and verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 command_synchronization`.
- [x] `net/minecraft/commands/synchronization/brigadier` (6 Java files): class-level Rust port tasks and parity tests are tracked in `CHECKLIST_JAVA_CLASSES.md` and verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 command_synchronization`.
- [ ] `net/minecraft/core` (38 Java files): add class-level Rust port tasks and parity tests.
- [x] `net/minecraft/core/cauldron` (3 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: `CauldronInteraction` and `CauldronInteractions` are modeled in `dispenser_cauldron` with Java-backed dispatcher, lookup, bucket, potion, bottle, cleaning, sound/stat/game-event, and side-effect tests, and package-info is audited as the nullability marker. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 cauldron_`.
- [ ] `net/minecraft/core/component` (12 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/core/component/predicates` (18 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/core/dispenser` (12 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/core/particles` (19 Java files): add class-level Rust port tasks and parity tests.
- [x] `net/minecraft/core/registries` (4 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: `BuiltInRegistries`, `ConcurrentHolderGetter`, and `Registries` are modeled in `registry::builtin` and `core_registries` with Java-backed built-in registry order/default/intrusive-holder/bootstrap/freeze/validation, cache, registry-key, path-helper, root-registry, and package-info tests. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 builtin_registry` and `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 core_registries`.
- [ ] `net/minecraft/data` (10 Java files): add class-level Rust port tasks and parity tests.
- [x] `net/minecraft/data/advancements` (3 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: `AdvancementProvider` and `AdvancementSubProvider` are modeled in `data_advancements` with Java-backed registry path, registry-future, subprovider, duplicate-id, stable-save, placeholder-factory, provider-name, and source-sentinel tests, and package-info is audited as the nullability marker. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 data_advancements`.
- [x] `net/minecraft/data/advancements/packs` (7 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: `VanillaAdvancementProvider` and the five vanilla advancement subproviders are modeled in `data_advancement_packs` with Java-backed subprovider order, advancement id lists, bundled JSON alignment, root/first/last sentinels, helper/entity/biome/count checks, and package-info is audited as the nullability marker. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 data_advancement_packs`.
- [x] `net/minecraft/data/info` (8 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: the seven data report providers are modeled in `data_info` with Java-backed report name/path, registry serialization, block/state/command/datapack/packet/component/registry output-shape, failure-sentinel, source-count, and source-sentinel tests, and package-info is audited as the nullability marker. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 data_info`.
- [x] `net/minecraft/data/loot` (5 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: `BlockLootSubProvider`, `EntityLootSubProvider`, `LootTableProvider`, and `LootTableSubProvider` are modeled in `data_loot`, `data_loot_entity`, and `data_loot_provider` with Java-backed loot helper, validation, provider/subprovider, stable-save, random-sequence, required-table, failure-text, and package-info tests. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 data_loot -- --nocapture`, `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 data_loot_entity -- --nocapture`, and `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 data_loot_provider -- --nocapture`.
- [ ] `net/minecraft/data/loot/packs` (17 Java files): add class-level Rust port tasks and parity tests.
- [x] `net/minecraft/data/metadata` (2 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: `PackMetadataGenerator` is modeled in `data_metadata` with Java-backed metadata-section, duplicate-key overwrite, `pack.mcmeta` output, provider-name, built-in pack-version, and feature-flag tests, and package-info is audited as the nullability marker. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 data_metadata`.
- [ ] `net/minecraft/data/recipes` (15 Java files): add class-level Rust port tasks and parity tests.
- [x] `net/minecraft/data/recipes/packs` (2 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: `VanillaRecipeProvider` is modeled in `data_vanilla_recipe_provider` with Java-backed provider/runner, feature-gated generation, smeltable groups, recipe-family counts, builder-family usage, custom/special/cooking/smithing sentinels, trim template, and source-drift tests, and package-info is audited as the nullability marker. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 vanilla_recipe_provider`.
- [x] `net/minecraft/data/registries` (5 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: `RegistriesDatapackGenerator`, `RegistryPatchGenerator`, `TradeRebalanceRegistries`, and `VanillaRegistries` are modeled in `data_registries` with Java-backed provider, patch lookup, trade-rebalance delegation, vanilla bootstrap order, biome-filter validation, source-count, and sentinel tests, and package-info is audited as the nullability marker. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 data_registries`.
- [x] `net/minecraft/data/structures` (5 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: `NbtToSnbt`, `SnbtDatafixer`, `SnbtToNbt`, and `StructureUpdater` are modeled in `data_structures` with Java-backed provider, folder-walk, extension filtering, SNBT/NBT conversion, datafix/update, write-cache, error-shape, source-count, and sentinel tests, and package-info is audited as the nullability marker. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 data_structures`.
- [ ] `net/minecraft/data/tags` (30 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/data/worldgen` (29 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/data/worldgen/biome` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/data/worldgen/features` (11 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/data/worldgen/placement` (11 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/gametest` (2 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/gametest/framework` (43 Java files): add class-level Rust port tasks and parity tests.
- [x] `net/minecraft/gizmos` (15 Java files): add class-level Rust port tasks and parity tests. — Covered by `src/gametest_resources/gizmo_models.rs` model parity for the full gizmo record/interface set plus `CHECKLIST_JAVA_CLASSES.md` per-file entries; verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 gizmo`.
- [x] `net/minecraft/locale` (3 Java files): add class-level Rust port tasks and parity tests. — Covered by `src/gametest_resources/locale_models.rs` and per-file entries in `CHECKLIST_JAVA_CLASSES.md` for `DeprecatedTranslationsInfo`, `Language`, and package metadata; verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 locale`.
- [ ] `net/minecraft/nbt` (36 Java files): add class-level Rust port tasks and parity tests.
- [x] `net/minecraft/nbt/visitors` (7 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: `CollectFields`, `CollectToTag`, `FieldSelector`, `FieldTree`, `SkipAll`, and `SkipFields` are modeled by `storage::nbt::tag_access` with Java-source sentinels for streaming visitor results, selector/tree construction, builder-stack collection, selected-field skipping, selected-field collection, missing-field accounting, and package metadata; verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 collect_fields_visitor`, `collect_to_tag_visitor`, `field_selector_and_tree`, `skip_all_visitor`, `skip_fields_visitor`, and `nbt_visitors_package_info`.
- [ ] `net/minecraft/network` (42 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/network/chat` (35 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/network/chat/contents` (10 Java files): add class-level Rust port tasks and parity tests.
- [x] `net/minecraft/network/chat/contents/data` (6 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: `BlockDataSource`, `DataSource`, `DataSources`, `EntityDataSource`, and `StorageDataSource` are modeled by `chat_component::nbt_contents::NbtDataSourceModel` with Java-source sentinels for the legacy `"source"` matcher, `"entity"`/`"block"`/`"storage"` codec ids, codec fields, NBT source conversion, missing-data behavior, and command-storage/entity/block lookup surfaces, while `package-info.java` is audited as metadata-only `@NullMarked`; verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 nbt_data_sources` and `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 chat_package_info`.
- [x] `net/minecraft/network/chat/contents/objects` (5 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: `AtlasSprite`, `ObjectInfo`, `ObjectInfos`, and `PlayerSprite` are modeled by `chat_component::object_contents`/`ObjectContent` with Java-source sentinels for object-info codecs, legacy `"object"` matching, `"atlas"`/`"player"` ids, font descriptions, default fallback strings, validation, and visitors, while `package-info.java` is audited as metadata-only `@NullMarked`; verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 object_contents` and `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 chat_package_info`.
- [x] `net/minecraft/network/chat/numbers` (7 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: blank, fixed, styled, type, and registry bootstrap number-format behavior is modeled in `chat_component::number_format` with Java-backed format/type/stream-id tests, and package-info is audited as the nullability marker. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 number_format` and `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 chat_package_info`.
- [x] `net/minecraft/network/codec` (7 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: `ByteBufCodecs`, `IdDispatchCodec`, `StreamCodec`, decoder/encoder/member-encoder interfaces, and package-info are modeled in `network::codec` with Java-backed primitive, optional, list/length, dispatch, combinator, duplicate/unknown-error, member-encoder-order, and source-sentinel tests. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 network::codec::tests`.
- [ ] `net/minecraft/network/protocol` (13 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/network/protocol/common` (23 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/network/protocol/common/custom` (4 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/network/protocol/configuration` (14 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/network/protocol/cookie` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/network/protocol/game` (194 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/network/protocol/handshake` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/network/protocol/login` (14 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/network/protocol/login/custom` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/network/protocol/ping` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/network/protocol/status` (8 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/network/syncher` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/package-info.java` (1 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/recipebook` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/references` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/resources` (15 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server` (27 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/advancements` (2 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/bossevents` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/chase` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/commands` (95 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/commands/data` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/dedicated` (7 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/dialog` (16 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/dialog/action` (7 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/dialog/body` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/dialog/input` (7 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/gui` (4 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/jsonrpc` (12 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/jsonrpc/api` (8 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/jsonrpc/dataprovider` (2 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/jsonrpc/internalapi` (18 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/jsonrpc/methods` (17 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/jsonrpc/security` (4 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/jsonrpc/websocket` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/level` (37 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/level/progress` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/network` (22 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/network/config` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/notifications` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/packs` (15 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/packs/linkfs` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/packs/metadata` (2 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/packs/metadata/pack` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/packs/repository` (11 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/packs/resources` (18 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/permissions` (12 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/players` (19 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/rcon` (4 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/rcon/thread` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/server/waypoints` (2 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/sounds` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/stats` (10 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/tags` (27 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util` (92 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/context` (4 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/datafix` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/datafix/fixes` (269 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/datafix/schemas` (117 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/debug` (19 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/debugchart` (8 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/eventlog` (4 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/filefix` (10 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/filefix/access` (11 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/filefix/fixes` (8 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/filefix/operations` (9 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/filefix/virtualfilesystem` (9 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/filefix/virtualfilesystem/exception` (9 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/monitoring/jmx` (2 Java files): add class-level Rust port tasks and parity tests.
- [x] `net/minecraft/util/parsing` (1 Java files): add class-level Rust port tasks and parity tests. — the package contains only `package-info.java`; its class row in `CHECKLIST_JAVA_CLASSES.md` is audited as a `@NullMarked` package marker with no runtime Rust behavior or parity test required.
- [ ] `net/minecraft/util/parsing/packrat` (14 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/parsing/packrat/commands` (14 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/profiling` (15 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/profiling/jfr` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/profiling/jfr/callback` (2 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/profiling/jfr/event` (13 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/profiling/jfr/parse` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/profiling/jfr/serialize` (2 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/profiling/jfr/stats` (14 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/profiling/metrics` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/profiling/metrics/profiling` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/profiling/metrics/storage` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/random` (4 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/thread` (9 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/valueproviders` (18 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/util/worldupdate` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world` (25 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/attribute` (21 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/attribute/modifier` (7 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/clock` (10 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/damagesource` (12 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/effect` (20 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity` (68 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/ai` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/ai/attributes` (9 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/ai/behavior` (104 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/ai/behavior/declarative` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/ai/behavior/warden` (10 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/ai/control` (9 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/ai/goal` (62 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/ai/goal/target` (11 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/ai/gossip` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/ai/memory` (8 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/ai/navigation` (7 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/ai/sensing` (27 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/ai/targeting` (2 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/ai/util` (8 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/ai/village` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/ai/village/poi` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/ambient` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/allay` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/armadillo` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/axolotl` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/bee` (2 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/camel` (4 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/chicken` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/cow` (8 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/dolphin` (2 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/equine` (13 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/feline` (7 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/fish` (8 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/fox` (2 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/frog` (8 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/goat` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/golem` (9 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/happyghast` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/nautilus` (8 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/panda` (2 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/parrot` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/pig` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/polarbear` (2 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/rabbit` (2 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/sheep` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/sniffer` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/squid` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/turtle` (2 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/animal/wolf` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/boss/enderdragon` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/boss/enderdragon/phases` (17 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/boss/wither` (2 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/decoration` (8 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/decoration/painting` (4 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/item` (4 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/monster` (24 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/monster/breeze` (8 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/monster/creaking` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/monster/hoglin` (4 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/monster/illager` (7 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/monster/piglin` (13 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/monster/skeleton` (7 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/monster/spider` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/monster/warden` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/monster/zombie` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/npc` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/npc/villager` (7 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/npc/wanderingtrader` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/player` (14 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/projectile` (12 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/projectile/arrow` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/projectile/hurtingprojectile` (7 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/projectile/hurtingprojectile/windcharge` (4 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/projectile/throwableitemprojectile` (9 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/raid` (4 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/schedule` (2 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/variant` (11 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/vehicle` (4 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/vehicle/boat` (7 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/entity/vehicle/minecart` (13 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/flag` (7 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/food` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/inventory` (61 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/inventory/tooltip` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/item` (101 Java files): add class-level Rust port tasks and parity tests.
- [x] `net/minecraft/world/item/alchemy` (5 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: `Potion`, `Potions`, `PotionContents`, and `PotionBrewing` are modeled in `item_alchemy` with source-backed registry/effect/color/brewing graph tests, and package-info is audited as the nullability marker. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 item_alchemy -- --nocapture`.
- [ ] `net/minecraft/world/item/component` (45 Java files): add class-level Rust port tasks and parity tests.
- [x] `net/minecraft/world/item/consume_effects` (7 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: `ConsumeEffect` and the five concrete consume-effect records are modeled in `item_consume_effects` with Java-backed dispatch-id, probability, status-effect, sound, and teleport behavior tests, and package-info is audited as the nullability marker. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 consume_effect`.
- [x] `net/minecraft/world/item/context` (4 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: `UseOnContext`, `BlockPlaceContext`, and `DirectionalPlaceContext` are modeled in `block_placement::PlaceContext` with Java relocation, direction ordering, dispenser directional placement, and source-backed tests; package-info is audited as the nullability marker. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_placement`.
- [ ] `net/minecraft/world/item/crafting` (52 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/item/crafting/display` (14 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/item/enchantment` (14 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/item/enchantment/effects` (26 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/item/enchantment/providers` (7 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/item/equipment` (8 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/item/equipment/trim` (7 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/item/slot` (12 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/item/trading` (11 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level` (53 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/biome` (19 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/block` (322 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/block/entity` (70 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/block/entity/trialspawner` (7 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/block/entity/vault` (7 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/block/grower` (2 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/block/piston` (7 Java files): add class-level Rust port tasks and parity tests.
- [x] `net/minecraft/world/level/block/sounds` (2 Java files): add class-level Rust port tasks and parity tests. — all package rows are covered in `CHECKLIST_JAVA_CLASSES.md`: `AmbientDesertBlockSoundsPlayer` is modeled in `block_sounds` with Java-backed random-gate, sound-id, dry-vegetation/sand tag, support-check, horizontal-column scan, vertical-scan, badlands skip, and planned-emission tests, and package-info is audited as the nullability marker. Verified by `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_sounds`.
- [ ] `net/minecraft/world/level/block/state` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/block/state/pattern` (4 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/block/state/predicate` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/block/state/properties` (32 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/border` (4 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/chunk` (31 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/chunk/status` (9 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/chunk/storage` (14 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/dimension` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/dimension/end` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/entity` (19 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/gameevent` (11 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/gameevent/vibrations` (4 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/gamerules` (7 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen` (43 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/blending` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/blockpredicates` (18 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/carver` (11 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/feature` (72 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/feature/configurations` (38 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/feature/featuresize` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/feature/foliageplacers` (14 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/feature/rootplacers` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/feature/stateproviders` (12 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/feature/treedecorators` (13 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/feature/trunkplacers` (12 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/flat` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/heightproviders` (9 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/material` (2 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/placement` (23 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/presets` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/structure` (19 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/structure/pieces` (7 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/structure/placement` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/structure/pools` (12 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/structure/pools/alias` (7 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/structure/structures` (32 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/structure/templatesystem` (33 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/structure/templatesystem/loader` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/structure/templatesystem/rule/blockentity` (7 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/levelgen/synth` (8 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/lighting` (15 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/material` (11 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/pathfinder` (15 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/portal` (4 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/redstone` (10 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/saveddata` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/saveddata/maps` (9 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/storage` (23 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/storage/loot` (15 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/storage/loot/entries` (16 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/storage/loot/functions` (49 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/storage/loot/parameters` (3 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/storage/loot/predicates` (25 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/storage/loot/providers/nbt` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/storage/loot/providers/number` (11 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/storage/loot/providers/score` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/timers` (6 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/level/validation` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/phys` (7 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/phys/shapes` (20 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/scores` (13 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/scores/criteria` (2 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/ticks` (14 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/timeline` (5 Java files): add class-level Rust port tasks and parity tests.
- [ ] `net/minecraft/world/waypoints` (9 Java files): add class-level Rust port tasks and parity tests.
