# Container Menu Checklist

## Relevant Java Source Files

- `decompiled-server-26.1.2/net/minecraft/world/inventory/AbstractContainerMenu.java` — base menu
- `decompiled-server-26.1.2/net/minecraft/world/inventory/Slot.java` — slot base
- `decompiled-server-26.1.2/net/minecraft/world/inventory/InventoryMenu.java` — player inventory
- `decompiled-server-26.1.2/net/minecraft/world/inventory/CraftingMenu.java` — 3×3 crafting
- `decompiled-server-26.1.2/net/minecraft/world/inventory/AbstractCraftingMenu.java` — crafting base
- `decompiled-server-26.1.2/net/minecraft/world/inventory/AbstractFurnaceMenu.java` — furnace base
- `decompiled-server-26.1.2/net/minecraft/world/inventory/FurnaceMenu.java` — furnace
- `decompiled-server-26.1.2/net/minecraft/world/inventory/BlastFurnaceMenu.java` — blast furnace
- `decompiled-server-26.1.2/net/minecraft/world/inventory/SmokerMenu.java` — smoker
- `decompiled-server-26.1.2/net/minecraft/world/inventory/AnvilMenu.java` — anvil
- `decompiled-server-26.1.2/net/minecraft/world/inventory/BeaconMenu.java` — beacon
- `decompiled-server-26.1.2/net/minecraft/world/inventory/BrewingStandMenu.java` — brewing stand
- `decompiled-server-26.1.2/net/minecraft/world/inventory/CartographyTableMenu.java` — cartography table
- `decompiled-server-26.1.2/net/minecraft/world/inventory/CrafterMenu.java` — crafter block
- `decompiled-server-26.1.2/net/minecraft/world/inventory/EnchantmentMenu.java` — enchanting table
- `decompiled-server-26.1.2/net/minecraft/world/inventory/GrindstoneMenu.java` — grindstone
- `decompiled-server-26.1.2/net/minecraft/world/inventory/LecternMenu.java` — lectern
- `decompiled-server-26.1.2/net/minecraft/world/inventory/LoomMenu.java` — loom
- `decompiled-server-26.1.2/net/minecraft/world/inventory/SmithingMenu.java` — smithing table
- `decompiled-server-26.1.2/net/minecraft/world/inventory/StonecutterMenu.java` — stonecutter
- `decompiled-server-26.1.2/net/minecraft/world/inventory/ChestMenu.java` — chest / double-chest
- `decompiled-server-26.1.2/net/minecraft/world/inventory/DispenserMenu.java` — dispenser/dropper
- `decompiled-server-26.1.2/net/minecraft/world/inventory/HopperMenu.java` — hopper
- `decompiled-server-26.1.2/net/minecraft/world/inventory/ShulkerBoxMenu.java` — shulker box
- `decompiled-server-26.1.2/net/minecraft/world/inventory/HorseInventoryMenu.java` — horse/llama
- `decompiled-server-26.1.2/net/minecraft/world/inventory/NautilusInventoryMenu.java` — happy ghast harness
- `decompiled-server-26.1.2/net/minecraft/world/inventory/MerchantMenu.java` — villager trading
- `decompiled-server-26.1.2/net/minecraft/world/inventory/MerchantContainer.java` — merchant container
- `decompiled-server-26.1.2/net/minecraft/world/inventory/MenuType.java` — menu type registry
- `RustCraft/src/inventory.rs` — RustCraft inventory
- `RustCraft/src/inventory_transactions.rs` — RustCraft inventory click handling
- `RustCraft/src/player_inventory.rs` — RustCraft player inventory

## AbstractContainerMenu Infrastructure

- [ ] Implement state-ID counter (`stateId`): incremented on every server-side slot change, sent in `ContainerSetSlot` / `ContainerSetContent` to detect client desync
- [ ] Implement `Slot` list: ordered slot registration, `Slot.getMaxStackSize()`, `Slot.mayPlace()`, `Slot.mayPickup()`, `Slot.onTake()`, `Slot.set()`, `Slot.isActive()`
- [ ] Implement carried-item stack (cursor): updated separately from slot contents
- [ ] Implement remote-slot shadow copies: `remoteSlots` list tracking last-synced client view; `sendAllDataToRemote()`, `sendSlotChange()`
- [ ] Implement `DataSlot` / `ContainerData` sync: integer data values (cook time, fuel time, enchant cost, etc.) sent via `ContainerSetData`
- [ ] Implement click validation: `ClickType` handling (PICKUP, QUICK_MOVE, SWAP, CLONE, THROW, QUICK_CRAFT, PICKUP_ALL) with server-side re-validation against current slot state
- [ ] Implement quick-craft (drag-split): track drag-start, drag-add slots, drag-end; split carried stack across valid target slots
- [ ] Implement `ContainerSynchronizer` and `ContainerListener` interfaces used to push updates back to client
- [ ] Implement `stillValid(player)` check on every tick (close menu if block/entity no longer accessible)
- [ ] Add parity test: stateId desync detection — client sending old stateId causes vanilla-compatible correction packet

## Player and Crafting Menus

- [ ] Implement `InventoryMenu`: 36 main slots, 4 armor slots, 1 offhand slot, 4 crafting input slots, 1 result slot; `RecipeBookMenu` integration; survival vs. creative inventory size difference; `ResultSlot` crafting trigger
- [ ] Implement `CraftingMenu`: 9 input slots, 1 result slot, `RecipeCraftingHolder` recipe matching update, `RecipeBook` unlock notification
- [ ] Implement `AbstractCraftingMenu`: shared crafting result logic, `slotsChanged()` triggering `RecipeManager.getResultFor()`, remainder items placed in grid
- [ ] Implement `RecipeBookMenu`: `RecipeBookType` per menu, recipe book state sync, recipe placement into grid on click
- [ ] Add parity test: 2×2 crafting grid result update on each slot change, recipe unlocking, remainder handling

## Furnace Menus

- [ ] Implement `AbstractFurnaceMenu`: input slot (0), fuel slot (1), result slot (2); `ContainerData` sync for `litTime`, `litDuration`, `cookingProgress`, `cookingTotalTime`; `FurnaceFuelSlot` slot type restriction; `FurnaceResultSlot` on-take XP release
- [ ] Implement `FurnaceMenu`: recipe book type `FURNACE`
- [ ] Implement `BlastFurnaceMenu`: recipe book type `BLAST_FURNACE`
- [ ] Implement `SmokerMenu`: recipe book type `SMOKER`
- [ ] Add parity test: fuel slot accepts only fuel items, result slot releases XP on extraction, progress data sync timing

## Workstation Menus

- [ ] Implement `AnvilMenu` (`ItemCombinerMenu`): input-left (0), input-right (1), result (2); rename-text input via custom payload; cost calculation (XP cost, `too expensive` threshold at cost ≥ 40 in survival); prior-work cost accumulation
- [ ] Implement `BeaconMenu`: payment slot, primary effect selector, secondary effect selector; `ContainerData` for beacon level; cost validation (accepts emerald/diamond/gold/iron/netherite/amethyst)
- [ ] Implement `BrewingStandMenu`: ingredient (3), fuel (4), potion slots (0-2); `ContainerData` for `brewTime` and `fuelLevel`
- [ ] Implement `CartographyTableMenu`: map input (0), paper/map (1), result (2); extend vs. clone vs. lock behavior based on input combination
- [ ] Implement `CrafterMenu`: 9 craftable grid slots + 1 result slot; per-slot enabled/disabled toggle via interaction; comparator output integration
- [ ] Implement `EnchantmentMenu`: item input (0), lapis input (1); `ContainerData` for 3 enchantment costs and 3 enchantment hints; require lapis and min-level check
- [ ] Implement `GrindstoneMenu`: input-left (0), input-right (1), result (2); disenchant + repair behavior; XP release on take proportional to removed enchantments
- [ ] Implement `LecternMenu`: book display, page-turn packet handling
- [ ] Implement `LoomMenu`: banner input (0), dye input (1), pattern item (2), result (3); available patterns filtered by banner patterns in registry + held pattern item
- [ ] Implement `SmithingMenu` (`ItemCombinerMenu`): template (0), base (1), addition (2), result (3); template validation, trim vs. transform recipe matching
- [ ] Implement `StonecutterMenu`: input (0), result (1); recipe list from `minecraft:stonecutting` type; selected-recipe index tracked
- [ ] Add parity test: anvil cost calculation for rename-only, repair, enchant-combine, and prior-work penalty
- [ ] Add parity test: enchanting table slot costs and enchantment hints for seeded items
- [ ] Add parity test: grindstone XP release proportional to enchantment cost removed

## Storage and Transfer Menus

- [ ] Implement `ChestMenu`: configurable row count (1–6), player inventory 27/36-slot prefix, chest inventory appended; double-chest = 54 slots
- [ ] Implement `DispenserMenu`: 3×3 grid (9 slots), player inventory appended
- [ ] Implement `HopperMenu`: 5-slot horizontal bar, player inventory appended
- [ ] Implement `ShulkerBoxMenu`: 27-slot grid, player inventory appended
- [ ] Implement `HorseInventoryMenu`: saddle slot (0), armor/chest slot (1), optional saddle-chest slots (2–16 for llama/horse-with-chest), player inventory appended
- [ ] Implement `AbstractMountInventoryMenu`: base for horse/llama inventory validation
- [ ] Implement `NautilusInventoryMenu` (new in 26.1.2): happy ghast harness inventory slots
- [ ] Add parity test: double-chest row count = 6, slot offset between chest and player inventory
- [ ] Add parity test: horse inventory saddle slot restriction (accepts only saddle item), armor slot restriction

## Merchant Menu

- [ ] Implement `MerchantMenu`: 3-slot `MerchantContainer` (input-left, input-right, result), player inventory appended; offer index tracking; `MerchantResultSlot` on-take side effects (decrement trade use count, XP grant to villager)
- [ ] Implement `MerchantContainer`: offer selection, `canTrade()` validation, `prepareTrade()` result slot update on input change
- [ ] Implement offer selection: clicking offer from offer list updates inputs; `selectOffer(index)` packet
- [ ] Implement demand mechanics: `priceMultiplier`, `demand`, `specialPrice` modification; `specialPrice` from hero-of-the-village effect
- [ ] Implement trade-use counting: `uses` increment, `maxUses`, `rewardExp` flag
- [ ] Implement restock: 2 times per day at workstation, resets `uses` toward `maxUses`
- [ ] Add Mineflayer merchant-menu test: select offer, verify result slot, shift-click trade, exhaust offer, observe restock timing
- [ ] Add parity test: demand multiplier increment after each purchase, price-restore after restock

## Container Menu Tests (Cross-Cutting)

- [ ] For every menu type: add normal click (left/right) test verifying slot content swap vs. split behavior
- [ ] For every menu type: add shift-click quick-move test verifying destination slot priority order matches vanilla
- [ ] For every menu type: add hotbar-swap (number key 1–9) test
- [ ] For every menu type: add drag-split test (left-click drag, right-click drag, middle-click drag creative)
- [ ] For every menu type: add double-click collect test (gather matching items into cursor)
- [ ] For every menu type: add drop (Q key) test inside and outside inventory window
- [ ] For every menu type: add creative-mode clone (middle click) test
- [ ] For every menu type: add carried-item mismatch correction test (client sends stale stateId, server corrects)
- [ ] For every menu type: add close-while-carrying test (cursor item drops on close)
- [ ] For every menu type: add disconnect-while-open test (inventory correctly drops/saves on disconnect)
- [ ] Add Mineflayer offline-mode window lifecycle test: open, click, close, reopen, disconnect mid-window for player inventory, chest, furnace, crafting table, anvil, and merchant menus

## Migrated From Main Checklist: Source-Derived Granularity Appendix - Container Menu Coverage

- [ ] Implement menu base behavior from `AbstractContainerMenu`: state IDs, slot lists, carried stack, remote slots, data slots, click validation, quick-craft tracking, synchronizers, listeners, and still-valid checks.
- [ ] Implement crafting/player menus: `InventoryMenu`, `CraftingMenu`, `AbstractCraftingMenu`, recipe book integration, 2x2 and 3x3 result updates, and result slot side effects.
- [ ] Implement furnace menus: `AbstractFurnaceMenu`, `FurnaceMenu`, `BlastFurnaceMenu`, `SmokerMenu`, progress data, fuel slot restrictions, and recipe-book categories.
- [ ] Implement workstation menus: `AnvilMenu`, `BeaconMenu`, `BrewingStandMenu`, `CartographyTableMenu`, `CrafterMenu`, `EnchantmentMenu`, `GrindstoneMenu`, `LecternMenu`, `LoomMenu`, `SmithingMenu`, and `StonecutterMenu`.
- [ ] Implement storage and transfer menus: `ChestMenu`, `DispenserMenu`, `HopperMenu`, `ShulkerBoxMenu`, `HorseInventoryMenu`, `AbstractMountInventoryMenu`, and `NautilusInventoryMenu`.
- [ ] Implement merchant menu behavior from `MerchantMenu`, `MerchantContainer`, and merchant result slots, including offer selection, demand, special price, XP, restock, and trade-use counting.
- [ ] Add Mineflayer merchant-menu tests for selecting offers, shift-click trading, rejected trades, stale offer IDs, XP bar updates, price changes, closing/reopening, and disconnecting mid-trade in offline mode.
- [ ] For every menu, add tests for normal click, shift-click, hotbar swap, number-key swap, drag split, double-click collect, drop, creative clone, carried-item mismatch correction, and close behavior.
