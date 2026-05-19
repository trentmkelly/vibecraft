# Items, Inventories, and Crafting Checklist

Item registry, stacks, components, inventories, creative mode, and crafting integration work moved out of the top-level checklist.

## Migrated From Main Checklist: Items, Inventories, And Crafting

- [ ] Implement all 101 top-level item classes represented under `net/minecraft/world/item`.
- [ ] Implement item registry IDs, components, max stack size, rarity, durability, repairability, use animation, cooldowns, food, equipment slots, enchantability, and tooltip behavior.
- [ ] Implement `ItemStack` and data component model.
- [ ] Implement inventory slots, carried item, cursor behavior, click actions, drag splitting, quick move, hotbar swap, drop, clone, and creative actions.
- [ ] Implement player inventory, armor, offhand, ender chest, containers, horse inventories, villager trading inventories, merchant offers, and crafting grids.
- [ ] Add Mineflayer equipment and inventory sync tests for armor, offhand, selected hotbar slot, item pickup, item drop, respawn retention rules, and disconnect/reconnect persistence.
- [x] Add raw 26.1.2 selected-inventory-slot sync fallback coverage that sends `serverbound/minecraft:set_carried_item`, persists the selected hotbar slot through disconnect/restart, and verifies reconnect emits the saved held-slot packet while armor/offhand/item-entity coverage waits on Mineflayer target-protocol support.
- [ ] Add a Mineflayer offline-mode inventory-login-persistence test that gives a bot items, disconnects before and after explicit save, reconnects, and verifies vanilla-compatible inventory restore timing.
- [ ] Add Mineflayer inventory correction tests that intentionally desync slot clicks, carried items, selected hotbar slot, creative actions, and drop packets in offline mode, then verify vanilla-compatible corrections.
- [x] Add raw 26.1.2 selected-hotbar correction fallback coverage that sends an out-of-range `serverbound/minecraft:set_carried_item`, restarts, and verifies reconnect keeps vanilla-compatible slot `0` rather than persisting the invalid client slot while broader inventory desync coverage waits on Mineflayer target-protocol support.
- [ ] Add a Mineflayer offline-mode login-inventory baseline test that verifies a fresh generated profile receives the vanilla empty inventory, selected slot, carried item, recipe book, and cursor state before any scripted action.
- [x] Add raw 26.1.2 login-inventory baseline fallback coverage that verifies a fresh generated offline profile receives `set_container_content` for the empty 46-slot player inventory, empty carried cursor item, selected hotbar slot 0, and initial chunk visibility while Mineflayer lacks target-protocol play support.
- [ ] Add Mineflayer offline-mode window lifecycle tests that open, click, close, reopen, disconnect mid-window, and reconnect for player inventory, chest, furnace, crafting table, anvil, and villager/trader-style menus.
- [ ] Implement recipe types, recipe book, unlocks, display data, and recipe serialization.
- [ ] Add a Mineflayer crafting/recipe-book test that unlocks recipes, crafts in 2x2 and 3x3 grids, opens a workstation, and verifies recipe sync and result slots against vanilla.
- [ ] Implement shaped, shapeless, smelting, blasting, smoking, campfire cooking, stonecutting, smithing, transmute, map cloning/extending, banner, shield, firework, suspicious stew, book cloning, repair, dyed item, and special recipes.
- [ ] Implement potion, tipped arrow, lingering potion, splash potion, cauldron, bottle, bucket, and fluid container behavior.
- [ ] Implement maps, compasses, clocks, recovery compasses, bundles, books, written books, knowledge books, spawn eggs, boats, minecarts, armor trims, smithing templates, shields, elytra, maces, tridents, bows, crossbows, fishing rods, shears, brushes, leads, name tags, music discs, ominous bottles, and trial keys.
- [x] Implement creative tabs and creative inventory packet behavior.
- [ ] Validate inventory transactions against vanilla packet traces.
