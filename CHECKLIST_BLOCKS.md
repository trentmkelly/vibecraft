# Blocks and Block States Checklist

Block registry, block-state, block behavior, and block-entity integration work moved out of the top-level checklist.

## Migrated From Main Checklist: Blocks, Block States, And Block Entities

- [ ] Implement all 322 top-level block classes represented under `net/minecraft/world/level/block`.
- [x] Implement block registry IDs, state definitions, properties, default states, map colors, sounds, hardness, resistance, collision, occlusion, light, pathfinding, and survival checks. — All data is probed from the official server jar (`tools/BlockPropertyDump.java`) and pinned 1:1: the 1168-block registry (fixing a 24-block copper bars/chain/lantern drift that shifted all ids ≥342) and full 29873-state table with cartesian property order/defaults in `block_states` (`tools/generate_block_states.py`, round-trip test against vendored `blocks.json`); per-state physics in `block_properties` (destroy speed, map colors, SoundType events, explosion resistance, friction/speed/jump, blocks-motion/occlusion/solid-render flags, light emission+dampening, 3-mode pathfindability, push reactions, fluid states, interned voxel shapes for all five channels, 18-bit isFaceSturdy×SupportType masks); lighting occlusion is exact via probed `Shapes.mergedFaceOccludes`/`faceShapeOccludes` matrices (`light_occlusion_26_1_2.json.gz`); and all 62 Java `canSurvive` overrides are ported in `block_survival` (dispatch on official block-type keys, data-driven `supports_*` tags via `block_tags`, coverage test enumerating every Java class). Live consumers (chunk palettes, instabreak, fluid gates, lighting) read these tables; the placement/destruction pipeline wiring is item #9.
- [ ] Implement block placement, replacement, rotation, mirroring, neighbor updates, shape updates, and destruction behavior.
- [ ] Add Mineflayer block place/break tests for offline-mode survival and creative bots, including reach distance, denied placement, placement orientation, drops, and visible block update acknowledgments.
- [ ] Add Mineflayer block interaction parity tests for right-click use, sneak-use, client prediction rollback, block entity update tags, and neighbor-shape updates after offline-mode placement.
- [ ] Add Mineflayer offline-mode block-entity interaction tests for signs, chests, furnaces, lecterns, bells, note blocks, campfires, cauldrons, spawners, and brushable blocks, verifying visible updates and vanilla-compatible denial paths.
- [ ] Extend Mineflayer offline-mode block-entity parity coverage to hanging signs, books, command blocks, skulls, banners, conduits, candles, vaults, trial spawners, calibrated sculk sensors, and chiseled bookshelves with explicit action coverage.
- [ ] Add Mineflayer offline-mode block-drop tests that break representative blocks with bare hand, correct tool, Silk Touch, Fortune, explosion damage, and `doTileDrops=false`, then compare inventory pickups and world item entities against official `server.jar`.
- [ ] Implement fluid interaction and waterlogging.
- [ ] Implement redstone: power propagation, dust, repeaters, comparators, observers, target blocks, sculk sensors, buttons, levers, pressure plates, tripwire, doors, trapdoors, pistons, and quasi-connectivity if observed.
- [ ] Implement gravity blocks.
- [ ] Implement plant growth, bonemeal, age, spread, saplings, fungi, crops, vines, leaves decay, moss, sculk, and coral.
- [ ] Implement fire spread, extinguishing, TNT ignition, explosions, and blast resistance. — PARTIAL (2026-06-11): fire *ignition* is live — `FlintAndSteelItem.useOn` (`item_flint_and_steel.rs` + `network/status/item_use_live.rs`) places `BaseFireBlock.getState` fire/soul-fire and lights campfires/candles/candle cakes 1:1. Still missing: the `FireBlock` scheduled tick (spread/age/burn-out/rain, `TODO(live-fire-tick)` in `item_use_live.rs`; the logic sketches in `fire.rs` are dead code and its `flammability()` table is heuristic, not the exact `FireBlock.bootstrap` igniteOdds/burnOdds registry), TNT ignition, and explosions. Nether-portal ignition via fire placement also waits on item #20 (`TODO(live-nether-portal)` in `item_flint_and_steel.rs`).
- [ ] Implement portals: nether portal search/create, end portal behavior, gateway behavior.
- [ ] Implement beds, respawn anchors, spawn points, and explosion behavior in invalid dimensions.
- [ ] Implement containers: chests, trapped chests, barrels, shulker boxes, hoppers, droppers, dispensers, furnaces, smokers, blast furnaces, brewing stands, crafters, decorated pots, lecterns, jukeboxes, beacons, enchanting tables, anvils, grindstones, smithing tables, looms, stonecutters, cartography tables, and crafting tables.
- [ ] Implement signs, hanging signs, books, command blocks, skulls, banners, conduits, bells, campfires, candles, cauldrons, note blocks, spawners, vaults, trial spawners, calibrated sculk sensors, chiseled bookshelves, and brushable blocks.
- [ ] Implement block entities, ticking block entities, serialization, update packets, and client data.
- [ ] Implement dispenser behaviors and cauldron interactions.
- [ ] Validate block behavior with placement/break/use/redstone regression tests against vanilla.
