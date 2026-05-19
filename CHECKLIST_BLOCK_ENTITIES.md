# Block Entity Checklist

## Relevant Java Source Files

- `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/` — all block entity implementations
- `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BlockEntity.java` — base class
- `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BlockEntityType.java` — type registry
- `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/AbstractFurnaceBlockEntity.java` — furnace base
- `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/FuelValues.java` — fuel registry
- `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/HopperBlockEntity.java` — hopper logic
- `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BeehiveBlockEntity.java` — beehive
- `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/CommandBlockEntity.java` — command block
- `decompiled-server-26.1.2/net/minecraft/world/inventory/` — container menus (tightly coupled)
- `decompiled-server-26.1.2/net/minecraft/world/level/block/` — block classes that pair with block entities
- `RustCraft/src/block_entity.rs` — RustCraft block entity implementation
- `RustCraft/src/container_block.rs` — RustCraft container block logic

## Base Block Entity Infrastructure

- [x] Implement `BlockEntity.getUpdateTag()` returning only the NBT subset sent in `BlockEntityData` network packets (not the full save NBT): `BlockEntity::get_update_tag()` returns empty update tags for container block entities and save-without-metadata tags for visible update entities; covered by `update_packets_use_position_type_and_update_tag`
- [x] Implement `BlockEntity.getUpdatePacket()` returning a `ClientboundBlockEntityDataPacket` at the correct packet ID: `BlockEntity::get_update_packet()` carries position, type, and update tag, and `network::play::ClientboundBlockEntityDataPacket` serializes the play packet shape
- [x] Implement `BlockEntity.changed()` marking the containing chunk as dirty for save: `BlockEntity::set_changed()` sets `changed` only after `set_level()`, covered by `changed_flag_only_sets_when_attached_to_level`
- [x] Implement `BlockEntity.clearRemoved()` / `setRemoved()` lifecycle hooks: `set_removed()` and `clear_removed()` update the removal flag, and ticking refuses removed block entities
- [x] Implement `BlockEntity.handleUpdateTag(tag)` applying the network-received subset on the simulated client side: `BlockEntity::handle_update_tag()` replaces custom data/components from the network tag while ignoring metadata fields (`id`, `x`, `y`, `z`); covered by `handle_update_tag_applies_network_subset_without_metadata`
- [x] Implement `BlockEntityTicker` dispatch: server-tick and client-tick registrations independently null-checked per type: `BlockEntity::tick(client_side)` checks `BlockEntityTypeInfo.tick_kind`, level attachment, and removal state; covered by `ticking_requires_level_side_match_and_not_removed`
- [x] Implement `BlockEntityType` registry with `validBlocks` set and version-compatible NBT deserialization via `DataFixer`: `BLOCK_ENTITY_TYPES` mirrors the 26.1.2 `BlockEntityType` surface including valid-block sets/op-only custom data, `BlockEntity::new()` and `load_static()` validate type/block compatibility, and `load_static_with_data_version()` gates NBT loading through the DataFixer compatibility check; covered by `block_entity_registry_matches_26_1_2_type_surface`, `validates_block_state_on_creation_and_load`, and `load_static_with_data_version_refuses_unsafe_migrations`
- [x] Implement `TickingBlockEntity` wrapper used by `ServerLevel` to schedule ticking block entities: `TickingBlockEntity` wraps a `BlockEntity`, exposes `tick()`, `is_removed()`, `pos()`, and `type_key()` for scheduler integration, and delegates side-aware ticking; covered by `ticking_block_entity_wrapper_exposes_scheduler_shape`
- [x] Add unit test: save NBT → load NBT round-trip preserves all fields for every block entity type: `save_load_round_trip_preserves_generic_fields_for_every_type` iterates all registered block entity types, serializes full metadata with custom data/components, reloads through `load_static()`, and verifies type, block state, payload maps, and transient flags
- [x] Add unit test: `getUpdateTag()` returns only the intended subset for each type (not more, not less): `update_tag_subset_is_stable_for_every_type` iterates the full block entity registry, asserts metadata fields never leak into update tags, verifies container entities send an empty tag, and verifies non-container entities preserve custom data/components

## Furnace Family

- [ ] Implement `AbstractFurnaceBlockEntity`: `litTime`/`litDuration` burn progress, `cookingProgress`/`cookingTotalTime` cook progress, per-tick recipe matching against the registered recipe type, `FuelValues` lookup for fuel slot items, XP float accumulation (`experience` field), XP orb release on item extraction, sided inventory access (`getMaxStackSize()` per slot), comparator output (0–14 based on progress/fuel)
- [ ] Implement `FurnaceBlockEntity`: recipe type `minecraft:smelting`, 200-tick cook time, standard fuels
- [ ] Implement `BlastFurnaceBlockEntity`: recipe type `minecraft:blasting`, 100-tick cook time (2× speed)
- [ ] Implement `SmokerBlockEntity`: recipe type `minecraft:smoking`, 100-tick cook time (2× speed)
- [ ] Add parity test: smelting progress ticks, fuel exhaustion, XP float accumulation, and XP orb count on item pickup
- [ ] Add parity test: blast furnace and smoker speed factors vs. furnace; invalid fuel items do not burn

## Container Block Entities

- [ ] Implement `BaseContainerBlockEntity`: custom-name `Component` component, lock `LockCode`, loot-table `ResourceKey`, loot-table seed, `unpackLootTable(player)` trigger on first open, `MenuProvider` interface with `createMenu()`
- [ ] Implement `RandomizableContainerBlockEntity`: loot-table seeded realization (loot table generates once, then replaced with real items)
- [ ] Implement `ChestBlockEntity`: lid animation via `ChestLidController` (0.1 per tick open/close), double-chest neighbor detection for merged access, `ContainerOpenersCounter` for hopper-blocking, comparator output
- [ ] Implement `TrappedChestBlockEntity`: emits `min(15, viewerCount)` redstone signal proportional to viewer count
- [ ] Implement `BarrelBlockEntity`: no lid animation, `ContainerOpenersCounter` for hopper-blocking, comparator output
- [ ] Implement `ShulkerBoxBlockEntity`: attached color, 4-state animation (CLOSED/OPENING/OPENED/CLOSING) at 0.1/tick, collision forced solid when CLOSED, 27-slot inventory
- [ ] Implement `DispenserBlockEntity`: 9-slot inventory, comparator output, behavior dispatch on redstone activation
- [ ] Implement `DropperBlockEntity`: 9-slot inventory, item-drop vs. dispense-into-container behavior difference
- [ ] Implement `HopperBlockEntity`: 8-tick transfer cooldown, suck-from-above (entity or block slot), push-below, `canPlaceItem()`/`canTakeItem()` sided checks, comparator output
- [x] Implement `ShelfBlockEntity` (new in 26.1.2): per-slot item display, 3-slot shelf storage (`MAX_ITEMS = 3`), comparator output based on filled-slot count
- [ ] Add parity test: chest loot-table realization (loot generated once, consistent across reconnect), double-chest merged access
- [ ] Add parity test: hopper 8-tick transfer rate, priority when both push and pull available, container hop chain
- [ ] Add parity test: trapped chest redstone signal level = viewer count
- [ ] Add Mineflayer randomizable-container test: open chests, barrels, dispensers, droppers, and shulker boxes before and after reconnect, verifying loot-table realization is idempotent

## Sign and Text Block Entities

- [ ] Implement `SignBlockEntity`: `SignText` for front face and back face independently, each with 4 `Component` lines, color (`DyeColor`), glowing state, editable state, wax-sealing state, `executeClickCommands(player)` permission check
- [ ] Implement `HangingSignBlockEntity`: same text model as `SignBlockEntity`, 6 different attachment types (ceiling, wall, log-wall variants)
- [x] Implement `LecternBlockEntity`: held `ItemStack` (written book), page index, `hasBook()`, `setBook()`, `clearContent()`, comparator output (0 = empty, 1–14 proportional to page/total pages, 15 on last page)
- [ ] Implement sign text filtering: route sign text through text-filter integration (same path as chat messages) before storing
- [ ] Add parity test: sign front/back text NBT serialization, waxed state preventing edits, glowing color tint
- [x] Add parity test: lectern comparator output across page transitions

## Utility Block Entities

- [x] Implement `BeaconBlockEntity`: pyramid-tier detection (scan 4 tiers, each layer must be diamond/emerald/gold/iron/netherite), primary effect (tier-dependent options), secondary effect (tier 4 only), fuel slot (payment item), beam color from stained glass above, `ContainerOpenersCounter` for GUI, comparator output (tier 0–4)
- [ ] Implement `BrewingStandBlockEntity`: ingredient slot (0), 3 potion output slots (1–3), fuel slot (4, blaze powder), brew-time countdown (400 ticks), fuel-count decrement, recipe matching (ingredient applies transformation per `PotionBrewing`)
- [ ] Implement `CrafterBlockEntity`: 9 crafting grid slots each with enabled/disabled flag, on-pulse crafting behavior (craft once per redstone leading-edge), comparator output (occupied non-disabled slots)
- [x] Implement `EnchantingTableBlockEntity`: bookshelf power scan (up to 15 bookshelves within range), visual book animation hint in `getUpdateTag()` (book open/close angle, page turn)
- [x] Implement `JukeboxBlockEntity`: disc `ItemStack`, `isPlaying` flag, `ticksSinceSongStarted`, comparator output (disc = signal from 1–15), `startPlaying()` / `stopPlaying()`, `SongPlayer` dispatch for jukebox song resource
- [x] Implement `ComparatorBlockEntity`: mode (COMPARE/SUBTRACT), `outputSignal` cached value, compare vs. subtract logic
- [x] Implement `DaylightDetectorBlockEntity`: sky-light level lookup, linear signal mapping (0–15), inverted mode (night sensor)
- [x] Implement `CommandBlockEntity`: command string, `lastOutput` component, `CommandBlockMode` (SEQUENCE/AUTO/REDSTONE), `isConditional`, `isAutomatic`, permission-level check, `performCommand(level)` execution
- [x] Add parity test: beacon tier detection with mixed pyramid materials, effect selection and duration
- [ ] Add parity test: brewing stand tick countdown, fuel consumption, ingredient slot cleared after brew

## World / System Block Entities

- [ ] Implement `SpawnerBlockEntity`: `SpawnData` (entity NBT, custom spawn rules), `nextSpawnData` pool (weighted random), `requiredPlayerRange` (16 blocks default), delay, `minSpawnDelay`/`maxSpawnDelay`, `spawnCount`, `maxNearbyEntities`, activation (player within range), mob-cap check, no-sky-access check
- [ ] Implement `TrialSpawnerBlockEntity`: state machine (INACTIVE → WAITING_FOR_PLAYERS → ACTIVE → WAITING_FOR_REWARD_EJECTION → COOLDOWN), normal vs. ominous config, `detectedPlayers` set per trial, ejection slot positions, `TrialSpawnerConfig` from data
- [x] Implement `JigsawBlockEntity`: joint type, target pool, name, final-state, selection priority, placement priority, orientation hint
- [x] Implement `StructureBlockEntity`: mode (SAVE/LOAD/CORNER/DATA), structure name, position offset, size, mirror, rotation, integrity, seed, show-bounding-box flag, `structureBlockSize` limits
- [x] Implement `TheEndGatewayBlockEntity`: age counter, exact-teleport flag (`exactTeleport`), exit position, beam animation (ray shot toward exit on teleport use)
- [x] Implement `TheEndPortalBlockEntity`: placeholder type entry in registry; triggers end dimension entry
- [x] Implement `TestBlockEntity` and `TestInstanceBlockEntity`: game-test framework support
- [ ] Add parity test: spawner delay and entity cycling after NBT save/load; mob-cap check prevents overcrowding
- [ ] Add parity test: trial spawner state machine transitions (activation, cooldown duration, ominous vs. normal config)

## Decorative / Lore Block Entities

- [x] Implement `BannerBlockEntity`: banner color, `BannerPatternLayers` list (up to 6 `BannerPatternLayer` each with pattern holder + `DyeColor`), serialization for both save NBT and `getUpdateTag()` network subset
- [x] Implement `BedBlockEntity`: empty placeholder (no data), required only for `BlockEntityType` registration and color variant
- [x] Implement `BrushableBlockEntity`: brush-progress animation (0–10), loot-table seeded item, loot-table seed, brush-item durability tracking, FULL/HIT/RESET state, `unpackLootTable()` on first brush
- [x] Implement `DecoratedPotBlockEntity`: `PotDecorations` (4 sides: back/left/right/front, each a sherd or brick), loot-table content, 1-slot item storage, wobble animation (type + ticks)
- [x] Implement `SkullBlockEntity`: owner profile component data, note-block sound data, custom-name component data, update tag, component stripping, and powered animation state
- [x] Implement `BellBlockEntity`: ringing-direction last value, resonating state, `ring(level, direction, entity)` trigger, bell-sound dispatch, entity highlighting on ring
- [x] Implement `CopperGolemStatueBlockEntity` (new in 26.1.2): weathering/waxed block-state mirroring, pose cycling, comparator output, and custom-name component preservation
- [x] Add parity test: banner pattern layers NBT field order, maximum 6 layers enforced, color+pattern round-trip
- [x] Add parity test: decorated pot wobble animation state after item insertion, destruction drops both sherds and stored item

## Mob / Environment Block Entities

- [ ] Implement `BeehiveBlockEntity`: occupants list (entity UUID, ticks-in-hive, min-occupation-ticks, has-nectar), anger flag, honey level (0–5), `addOccupant()`, `emptyAllLivingFromHive()`, `releaseAllOccupants()`, hive-full check, fire-destroy behavior
- [x] Implement `ConduitBlockEntity`: frame block scanning (prismarine variants), active condition threshold (16 blocks minimum), target tracking within 8 blocks, attack-cooldown of 40 ticks, conduit-power effect radius (16–96 blocks based on frame count)
- [x] Implement `CampfireBlockEntity`: 4 cooking slots with independent `cookingTime` (600 tick default) and `cookingProgress`, signal-fire flag (hay bale below doubles smoke height), lit state determines ticking
- [ ] Implement `SculkSensorBlockEntity`: vibration listener phase (LISTENING/TICKING/VIBRATION_DONE), vibration event data (source position, distance, vibration type), last-vibration frequency (output signal), delay-emit countdown, `Listening` and `Vibrating` phases
- [ ] Implement `CalibratedSculkSensorBlockEntity`: filter frequency from comparator input side; only propagate vibration events matching the filter frequency
- [ ] Implement `SculkCatalystBlockEntity`: charge queue on death of nearby mobs (charge proportional to XP), sculk spread emission toward charge targets
- [ ] Implement `SculkShriekerBlockEntity`: warning level accumulator (0–3), shriek cooldown, darkness-required check, `tryShriek(player)` with warden spawn check at level 3
- [ ] Implement `CreakingHeartBlockEntity`: linked creaking entity UUID, active/inactive state per time-of-day, phase tracking for creaking activation/deactivation
- [ ] Add parity test: sculk sensor vibration delay, frequency output for each `GameEvent` type
- [ ] Add parity test: sculk shrieker warning level accumulation, 10 s cooldown between shrieks, warden spawn at level 3
- [ ] Add parity test: beehive occupancy persistence, honey level increment after occupant leaves with nectar

## Block Entity Tests (Cross-Cutting)

- [ ] For every block entity type: add placement + immediate chunk-unload/reload round-trip test verifying NBT field identity
- [ ] For every block entity type: add `getUpdateTag()` / `BlockEntityData` packet field subset test (only expected fields present)
- [ ] For every tickable block entity: add tick-driven state transition test (e.g., furnace burn progress, spawner delay countdown, campfire cook progress)
- [ ] For every GUI-bearing block entity: add open-menu / container-id / close test verifying correct `MenuType` and initial slot contents
- [ ] For every block entity with a comparator output: add signal level test across all boundary states
- [ ] For every block entity: add `/data get block` NBT access test via command model or unit test
- [ ] For every block entity: add destruction drop test (correct tool, Silk Touch, explosion, correct drops including stored items)

## Migrated From Main Checklist: Source-Derived Granularity Appendix - Block Entity Coverage

- [x] Implement base `BlockEntity`, ticker dispatch, type registry, update tag, update packet, save/load, and chunk attachment behavior.
- [ ] Implement furnace family block entities: `AbstractFurnaceBlockEntity`, `FurnaceBlockEntity`, `BlastFurnaceBlockEntity`, and `SmokerBlockEntity`, including burn time, cook time, fuel values, recipe matching, XP storage, sided inventory, lit state, and comparator output.
- [ ] Implement container block entities: `BaseContainerBlockEntity`, `RandomizableContainerBlockEntity`, `ChestBlockEntity`, `TrappedChestBlockEntity`, `BarrelBlockEntity`, `ShulkerBoxBlockEntity`, `DispenserBlockEntity`, `DropperBlockEntity`, `HopperBlockEntity`, and `ShelfBlockEntity`, including loot tables, custom names, locks, viewer counts, lid animation, sided access, and redstone/comparator interactions.
- [ ] Add Mineflayer randomizable-container tests that open generated chests, barrels, dispensers, droppers, and shulker boxes before and after reconnect, verifying loot-table realization happens once, custom names/locks are enforced, and comparator-visible contents match vanilla.
- [ ] Implement sign and text block entities: `SignBlockEntity`, `HangingSignBlockEntity`, `LecternBlockEntity`, and book/sign filtering, editing, waxed state, front/back text, click commands, and command execution permissions.
- [ ] Implement utility block entities: `BeaconBlockEntity`, `BrewingStandBlockEntity`, `CrafterBlockEntity`, `EnchantingTableBlockEntity`, `JukeboxBlockEntity`, `ComparatorBlockEntity`, `DaylightDetectorBlockEntity`, and `CommandBlockEntity`.
- [ ] Implement world/system block entities: `SpawnerBlockEntity`, `TrialSpawnerBlockEntity`, `JigsawBlockEntity`, `StructureBlockEntity`, `TheEndGatewayBlockEntity`, `TheEndPortalBlockEntity`, `TestBlockEntity`, and `TestInstanceBlockEntity`.
- [x] Implement decorative/lore block entities: `BannerBlockEntity`, `BedBlockEntity`, `BrushableBlockEntity`, `DecoratedPotBlockEntity`, `SkullBlockEntity`, `BellBlockEntity`, and `CopperGolemStatueBlockEntity`.
- [ ] Implement mob/environment block entities: `BeehiveBlockEntity`, `ConduitBlockEntity`, `CampfireBlockEntity`, `SculkSensorBlockEntity`, `CalibratedSculkSensorBlockEntity`, `SculkCatalystBlockEntity`, `SculkShriekerBlockEntity`, and `CreakingHeartBlockEntity`.
- [ ] For every block entity, add tests for placement, ticking, GUI open/close, save/load, chunk unload/reload, comparator output, network update tag, `/data` command access, and destruction drops.
