# Blocks and Block States Checklist

Block registry, block-state, block behavior, and block-entity integration work moved out of the top-level checklist.

## Migrated From Main Checklist: Blocks, Block States, And Block Entities

- [ ] Implement all 322 top-level block classes represented under `net/minecraft/world/level/block`.
- [ ] Implement block registry IDs, state definitions, properties, default states, map colors, sounds, hardness, resistance, collision, occlusion, light, pathfinding, and survival checks.
- [ ] Implement block placement, replacement, rotation, mirroring, neighbor updates, shape updates, and destruction behavior.
- [ ] Add Mineflayer block place/break tests for offline-mode survival and creative bots, including reach distance, denied placement, placement orientation, drops, and visible block update acknowledgments.
- [ ] Add Mineflayer block interaction parity tests for right-click use, sneak-use, client prediction rollback, block entity update tags, and neighbor-shape updates after offline-mode placement.
- [ ] Add Mineflayer offline-mode block-entity interaction tests for signs, chests, furnaces, lecterns, bells, note blocks, campfires, cauldrons, spawners, and brushable blocks, verifying visible updates and vanilla-compatible denial paths.
- [ ] Add Mineflayer offline-mode block-drop tests that break representative blocks with bare hand, correct tool, Silk Touch, Fortune, explosion damage, and `doTileDrops=false`, then compare inventory pickups and world item entities against official `server.jar`.
- [ ] Implement fluid interaction and waterlogging.
- [ ] Implement redstone: power propagation, dust, repeaters, comparators, observers, target blocks, sculk sensors, buttons, levers, pressure plates, tripwire, doors, trapdoors, pistons, and quasi-connectivity if observed.
- [ ] Implement gravity blocks.
- [ ] Implement plant growth, bonemeal, age, spread, saplings, fungi, crops, vines, leaves decay, moss, sculk, and coral.
- [ ] Implement fire spread, extinguishing, TNT ignition, explosions, and blast resistance.
- [ ] Implement portals: nether portal search/create, end portal behavior, gateway behavior.
- [ ] Implement beds, respawn anchors, spawn points, and explosion behavior in invalid dimensions.
- [ ] Implement containers: chests, trapped chests, barrels, shulker boxes, hoppers, droppers, dispensers, furnaces, smokers, blast furnaces, brewing stands, crafters, decorated pots, lecterns, jukeboxes, beacons, enchanting tables, anvils, grindstones, smithing tables, looms, stonecutters, cartography tables, and crafting tables.
- [ ] Implement signs, hanging signs, books, command blocks, skulls, banners, conduits, bells, campfires, candles, cauldrons, note blocks, spawners, vaults, trial spawners, calibrated sculk sensors, chiseled bookshelves, and brushable blocks.
- [ ] Implement block entities, ticking block entities, serialization, update packets, and client data.
- [ ] Implement dispenser behaviors and cauldron interactions.
- [ ] Validate block behavior with placement/break/use/redstone regression tests against vanilla.
