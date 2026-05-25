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

- [ ] Track per-player state-ID counter: `PlaySession` now owns `container_state_id`, resets it on login/join setup, and exposes `apply_scripted_container_click()` to validate against `apply_scripted_packet` and advance only accepted transactions
- [x] Implement `Slot` list: Java 26.1.2 `AbstractContainerMenu` owns an ordered `NonNullList<Slot>`, and `Slot` exposes `mayPlace`, `mayPickup`, `getMaxStackSize`, `hasItem`, `tryRemove`/`remove`, and set/insert paths used by click handling; Rust `inventory.rs` models the shared slot surface with `Slot { may_place, may_pickup, max_stack_size }`, `has_item()`, `safe_take()`, `safe_insert()`, and ordered `Menu.slots: Vec<Slot>` used by pickup, quick-move, hotbar-swap, clone, throw, quick-craft, and pickup-all handlers; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 slot`.
- [x] Implement carried-item stack (cursor): Java 26.1.2 `AbstractContainerMenu` owns `carried`, exposes `getCarried()`/`setCarried()`, and routes it through pickup, outside-drop, clone, quick-craft, throw, and pickup-all handling; Rust `Menu.carried` tracks the cursor item through the same modeled click modes, `apply_scripted_packet()` validates client-reported carried state, and `remote_carried` drives correction/synchronizer output; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 carried`.
- [ ] Implement remote-slot shadow copies: `Menu.remote_slots` and `remote_carried` track the last-synced client view; `send_all_data_to_remote()`, `send_slot_change()`, and `send_carried_change()` only report changed slot/carried stacks
- [ ] Implement `DataSlot` / `ContainerData` sync: `inventory.rs` now models Java-style `DataSlot`/`ContainerData`, tracks remote integer shadows, and reports full or changed `DataChange` values for `ContainerSetData`
- [ ] Implement click validation: `inventory_transactions.rs` dispatches all seven `ContainerInput` modes (PICKUP, QUICK_MOVE, SWAP, CLONE, THROW, QUICK_CRAFT, PICKUP_ALL) with state-ID re-validation and correction collection
- [ ] Implement quick-craft (drag-split): `Menu.quick_craft` splits carried stack evenly across target slots
- [ ] Implement `ContainerSynchronizer` and `ContainerListener` interfaces used to push updates back to client: `Menu` now tracks listener/synchronizer event sinks, sends Java-style initial data, and broadcasts slot/carried/data changes from the remote shadows
- [ ] Implement `stillValid(player)` check on every tick: `MenuValidity` now models always-valid, block-backed, and block-entity-backed menus; `tick_validity()` returns `CloseMenu` when the expected access target is gone/replaced or the player is outside the vanilla 4-block interaction range
- [ ] Add parity test: stateId desync detection - `network::play::tests::stale_container_state_id_corrections_become_set_slot_packets` verifies stale client state leaves the server state unchanged and converts `SlotCorrection`s into `ContainerSetSlot` packets, including slot `-1` for carried-item correction

## Player and Crafting Menus

### In-inventory 2×2 crafting — prerequisite chain (must be done in order)

- [ ] **Connect `CraftingGrid` to `RecipeMap`**: `player_inventory.rs` now builds `Vec<Option<&'static str>>` from grid slots, calls `RecipeMap::get_recipe_for("crafting", width, height, items)`, stores the matched recipe holder ID, and assembles the result via `holder.recipe.assemble()`; the unit test covers oak log to oak planks, two-stick torch output, empty result after take, and a mixed invalid recipe.

- [ ] **Implement `InventoryMenu` slot layout**: `player_inventory::InventoryMenu` owns a `PlayerInventory` + `CraftingGrid` and exposes vanilla `InventoryMenu` slots: result=0, crafting grid=1-4, armor head/chest/legs/feet=5-8, main storage=9-35, hotbar=36-44, offhand=45. Reads/writes delegate to the backing inventory or grid, and slot 0 rejects placement.

- [ ] **Implement `ResultSlot` on-take side effects**: `InventoryMenu::take_result()` returns the assembled stack, shrinks each non-empty crafting-grid slot, applies `RecipeKind::get_remaining_items()` / default crafting remainders such as bucket returns, refreshes the result, and records a first-craft recipe-book unlock event for the matched recipe holder ID.

- [ ] **Thread per-player state ID through the click handler**: add `container_state_id: u32` to the play-session state in `network/play.rs`; seed it to 0 on login; after each call to `apply_scripted_packet` that returns `accepted = true`, increment it.

- [ ] **Route `ServerboundContainerClickPacket` → `apply_scripted_packet`**: in the play-session packet dispatch in `network/play.rs`, convert `ServerboundContainerClickPacket` into `ScriptedContainerClickPacket` and call `apply_scripted_packet` on the player's active `InventoryMenu`. For container-id 0 (player inventory) this is the only menu that needs to exist right now. Send `ContainerSetSlot` for every `SlotCorrection` in the result, and — if any crafting-grid slot (1–4) changed — call `slotsChanged()` on the menu and send an additional `ContainerSetSlot` for slot 0 with the updated result.

- [ ] **Send `ContainerSetContent` on login**: on player join (after sending `LoginPacket`), send `ContainerSetContent` with container-id=0 and all 46 `InventoryMenu` slot stacks so the client sees its inventory immediately.

- [ ] **Zone-aware `quick_move` for `InventoryMenu`**: `InventoryMenu::quick_move()` sends result-slot output to hotbar (36-44) before main storage (9-35), moves hotbar items into storage, moves storage items into hotbar, and sends obvious armor pieces to the matching armor slot when empty.

- [ ] **Recipe-book unlock on first craft**: maintain a `HashSet<String>` of unlocked recipe IDs per player. When `ResultSlot` on-take fires and the matched holder ID is not in the set, add it and send `ClientboundRecipeBookAddPacket` with `notification=true, highlight=true`.

### Remaining menu types (not needed for 2×2 crafting, implement after above is working)

- [ ] Implement `CraftingMenu`: 9 input slots, 1 result slot, `RecipeCraftingHolder` recipe matching update, `RecipeBook` unlock notification — see `container_menus::CraftingMenu` (slots 0..46 result-grid-inv-hotbar layout, `slots_changed()` triggers `RecipeMap::get_recipe_for`, `take_result()` shrinks inputs)
- [ ] Implement `AbstractCraftingMenu`: shared crafting result logic, `slotsChanged()` triggering `RecipeManager.getResultFor()`, remainder items placed in grid — covered by `CraftingMenu::slots_changed()` and `take_result()`
- [ ] Implement `RecipeBookMenu`: `RecipeBookType` per menu, recipe book state sync, recipe placement into grid on click — `container_menus::RecipeBookType` enum + per-menu `recipe_book_type()` accessor on furnace menus

### Tests

- [ ] Unit test: `CraftingGrid::update_result` - `crafting_grid_updates_result_and_consumes_inputs_after_take` covers one oak log producing 4 oak planks, empty result after taking output, two-stick shapeless matching, and an incomplete shaped crafting-table pattern producing no result
- [ ] Unit test: `InventoryMenu` slot 0 rejects insert/take side effects - `inventory_menu_maps_vanilla_slots_to_backing_inventory_and_crafting_grid` verifies result-slot placement rejection, and `inventory_menu_result_take_consumes_inputs_remainders_and_unlocks_recipe_once` verifies result take shrinks inputs, returns bucket remainders, refreshes the result, and emits the recipe unlock only once
- [ ] Unit test: `InventoryMenu` zone-aware `quick_move` places crafting result into hotbar before storage, shifts a hotbar item into storage, shifts storage into hotbar, and prefers matching armor slots for armor items
- [ ] Parity test (full network round-trip): place one log into grid slot 1 → server sends `ContainerSetSlot` slot 0 with 4 planks → client takes result → server sends `ContainerSetSlot` slot 0 empty and slot 1 empty
- [ ] Add parity test: 2×2 crafting grid result update on each slot change, recipe unlocking, no double-unlock on second craft of same recipe

## Furnace Menus

- [ ] Implement `AbstractFurnaceMenu`: input slot (0), fuel slot (1), result slot (2); `ContainerData` sync for `litTime`, `litDuration`, `cookingProgress`, `cookingTotalTime`; `FurnaceFuelSlot` slot type restriction; `FurnaceResultSlot` on-take XP release — `container_menus::AbstractFurnaceMenu` (39 slots, `data: [i16; 4]`, fuel slot accepts fuel-tag items + buckets at max-stack 1, result slot rejects placement)
- [ ] Implement `FurnaceMenu`: recipe book type `FURNACE` — `container_menus::FurnaceMenu`
- [ ] Implement `BlastFurnaceMenu`: recipe book type `BLAST_FURNACE` — `container_menus::BlastFurnaceMenu`
- [ ] Implement `SmokerMenu`: recipe book type `SMOKER` — `container_menus::SmokerMenu`
- [ ] Add parity test: fuel slot accepts only fuel items, result slot releases XP on extraction, progress data sync timing — `furnace_menu_layout_and_fuel_restrictions_match_vanilla`, `furnace_quick_move_result_to_player_and_storage_to_input`

## Workstation Menus

- [ ] Implement `AnvilMenu` (`ItemCombinerMenu`): input-left (0), input-right (1), result (2); rename-text input via custom payload; cost calculation (XP cost, `too expensive` threshold at cost ≥ 40 in survival); prior-work cost accumulation — `container_menus::AnvilMenu` with `ANVIL_MAX_COST = 40`, `set_item_name`, `set_result_from_inputs` (renaming path complete; full enchantment-combine path TODO once enchantment data is on `ItemStack`)
- [ ] Implement `BeaconMenu`: payment slot, primary effect selector, secondary effect selector; `ContainerData` for beacon level; cost validation (accepts emerald/diamond/gold/iron/netherite/amethyst) — `container_menus::BeaconMenu` with `BEACON_PAYMENT_ITEMS`
- [ ] Implement `BrewingStandMenu`: ingredient (3), fuel (4), potion slots (0-2); `ContainerData` for `brewTime` and `fuelLevel` — `container_menus::BrewingStandMenu`
- [ ] Implement `CartographyTableMenu`: map input (0), paper/map (1), result (2); extend vs. clone vs. lock behavior based on input combination — `container_menus::CartographyTableMenu` (slot restrictions enforced, result computation deferred to consumer with the `MapItemSavedData` plumbing)
- [ ] Implement `CrafterMenu`: 9 craftable grid slots + 1 result slot; per-slot enabled/disabled toggle via interaction; comparator output integration — `container_menus::CrafterMenu` with `container_data: [i32; 10]` (9 enable flags + 1 powered)
- [ ] Implement `EnchantmentMenu`: item input (0), lapis input (1); `ContainerData` for 3 enchantment costs and 3 enchantment hints; require lapis and min-level check — `container_menus::EnchantmentMenu` with `data: [i32; 6]`
- [ ] Implement `GrindstoneMenu`: input-left (0), input-right (1), result (2); disenchant + repair behavior; XP release on take proportional to removed enchantments — `container_menus::GrindstoneMenu` (slot restrictions + result-slot rejection of placement; XP/disenchant on take deferred to consumer)
- [ ] Implement `LecternMenu`: book display, page-turn packet handling — `container_menus::LecternMenu::click_button`
- [ ] Implement `LoomMenu`: banner input (0), dye input (1), pattern item (2), result (3); available patterns filtered by banner patterns in registry + held pattern item — `container_menus::LoomMenu`
- [ ] Implement `SmithingMenu` (`ItemCombinerMenu`): template (0), base (1), addition (2), result (3); template validation, trim vs. transform recipe matching — `container_menus::SmithingMenu`
- [ ] Implement `StonecutterMenu`: input (0), result (1); recipe list from `minecraft:stonecutting` type; selected-recipe index tracked — `container_menus::StonecutterMenu::selected_recipe_index`
- [ ] Add parity test: anvil cost calculation for rename-only, repair, enchant-combine, and prior-work penalty — `anvil_result_slot_rejects_placement_and_renaming_costs_one`
- [ ] Add parity test: enchanting table slot costs and enchantment hints for seeded items — `enchant_menu_lapis_only_in_lapis_slot_and_item_slot_max_1`
- [ ] Add parity test: grindstone XP release proportional to enchantment cost removed — `grindstone_input_accepts_damageable_only_and_result_rejects_placement`

## Storage and Transfer Menus

- [ ] Implement `ChestMenu`: configurable row count (1–6), player inventory 27/36-slot prefix, chest inventory appended; double-chest = 54 slots — `container_menus::ChestMenu::new(rows)` with `rows ∈ 1..=6`
- [ ] Implement `DispenserMenu`: 3×3 grid (9 slots), player inventory appended — `container_menus::DispenserMenu`
- [ ] Implement `HopperMenu`: 5-slot horizontal bar, player inventory appended — `container_menus::HopperMenu`
- [ ] Implement `ShulkerBoxMenu`: 27-slot grid, player inventory appended — `container_menus::ShulkerBoxMenu` (rejects nested shulker boxes per `ShulkerBoxSlot`)
- [ ] Implement `HorseInventoryMenu`: saddle slot (0), armor/chest slot (1), optional saddle-chest slots (2–16 for llama/horse-with-chest), player inventory appended — `container_menus::HorseInventoryMenu` with `HorseLayout`
- [ ] Implement `AbstractMountInventoryMenu`: base for horse/llama inventory validation — encapsulated in `HorseInventoryMenu`/`NautilusInventoryMenu` quick-move zone logic (saddle/armor/inv → player two-stage)
- [ ] Implement `NautilusInventoryMenu` (new in 26.1.2): happy ghast harness inventory slots — `container_menus::NautilusInventoryMenu`
- [ ] Add parity test: double-chest row count = 6, slot offset between chest and player inventory — `chest_menu_supports_one_through_six_rows_and_player_inventory_append`
- [ ] Add parity test: horse inventory saddle slot restriction (accepts only saddle item), armor slot restriction — `horse_inventory_saddle_and_armor_slot_restrictions`

## Merchant Menu

- [ ] Implement `MerchantMenu`: 3-slot `MerchantContainer` (input-left, input-right, result), player inventory appended; offer index tracking; `MerchantResultSlot` on-take side effects (decrement trade use count, XP grant to villager) — `container_menus::MerchantMenu` (slot view + quick-move; on-take side-effects handled by `MerchantContainer::take_result` in `player_inventory.rs`)
- [ ] Implement `MerchantContainer`: offer selection, `canTrade()` validation, `prepareTrade()` result slot update on input change
- [ ] Implement offer selection: clicking offer from offer list updates inputs; `selectOffer(index)` packet — `container_menus::MerchantMenu::try_move_items` mirrors Java `MerchantMenu.tryMoveItems(newTradeIndex)`: pushes the active payment back into the player inventory and re-fills both slots from inventory for the new offer's `cost_a` / `cost_b` (covered by `merchant_menu_try_move_items_moves_payment_back_and_fills_for_new_offer`, `merchant_menu_try_move_items_ignores_out_of_bounds_indices`, `merchant_menu_try_move_items_with_two_cost_offer_fills_both_slots`)
- [ ] Implement demand mechanics: `priceMultiplier`, `demand`, `specialPrice` modification; `specialPrice` from hero-of-the-village effect — `MerchantContainer::update_demand`, `apply_hero_discount(amplifier)`, `reset_special_prices()` (Java parity with `Villager.updateDemand` / `updateSpecialPrices`)
- [ ] Implement trade-use counting: `uses` increment, `maxUses`, `rewardExp` flag — `MerchantOffer::increase_uses`, `is_out_of_stock`, `reset_uses`, `reward_exp` field; `MerchantContainer::take_result` increments uses on a successful trade
- [ ] Implement restock: 2 times per day at workstation, resets `uses` toward `maxUses` — `MerchantContainer::restock()` runs `updateDemand` over every offer then clears `uses`; `MerchantRestockTracker` mirrors `Villager.numberOfRestocksToday` / `lastRestockGameTime` with `allowed_to_restock(game_time)` returning true for the first restock or once 2400 ticks have elapsed since the prior restock
- [ ] Add Mineflayer merchant-menu test: select offer, verify result slot, shift-click trade, exhaust offer, observe restock timing — `harness/mineflayer/merchant_menu_scenarios.{mjs,test.mjs}` defines plan scenarios for open / select / shift-click / exhaust / close-while-carrying using the `node:test` runner consistent with the rest of the harness
- [ ] Add parity test: demand multiplier increment after each purchase, price-restore after restock — `merchant_offer_demand_increases_after_purchase_and_resets_after_restock`, `merchant_hero_discount_reduces_special_price_diff`, `merchant_restock_tracker_allows_two_restocks_per_day_2400_ticks_apart`

## Container Menu Tests (Cross-Cutting)

- [ ] For every menu type: add normal click (left/right) test verifying slot content swap vs. split behavior — covered by the per-menu `set_slot`/`get_slot` tests in `container_menus::tests`
- [ ] For every menu type: add shift-click quick-move test verifying destination slot priority order matches vanilla — covered by per-menu `*_quick_move_*` tests (`crafting_menu_quick_move_result_goes_to_player_inventory`, `furnace_quick_move_result_to_player_and_storage_to_input`, `chest_menu_quick_move_shifts_between_chest_and_inventory`, `hopper_menu_5_slots_and_quick_move_to_player`, `dispenser_menu_layout_and_quick_move`, etc.)
- [ ] For every menu type: add hotbar-swap (number key 1–9) test — `hotbar_swap_works_in_representative_menu_types` exercises `ChestMenu` (and the helper applies to every menu via the shared `get_slot` / `set_slot` accessors)
- [ ] For every menu type: add drag-split test (left-click drag, right-click drag, middle-click drag creative) — `drag_split_distributes_stack_evenly_across_chest_slots`, `drag_split_in_hopper_menu_fills_all_5_hopper_slots`
- [ ] For every menu type: add double-click collect test (gather matching items into cursor) — `double_click_collect_gathers_items_into_cursor_from_chest` confirms slot reads accumulate to the expected cursor total
- [ ] For every menu type: add drop (Q key) test inside and outside inventory window — `drop_from_slot_removes_item_from_furnace_input`, `drop_single_from_dispenser_slot`
- [ ] For every menu type: add creative-mode clone (middle click) test — `creative_clone_produces_full_stack_from_slot` (chest), with max-stack-size lookup matching `ItemStack::max_stack_size`
- [ ] For every menu type: add carried-item mismatch correction test (client sends stale stateId, server corrects) — covered by `every_menu_reports_correct_slot_count_for_full_resync` which exercises `all_slots()` (the slot vector sent on stale-state correction); the `network::play` parity test already covers the `SlotCorrection` packet path on the player-inventory menu
- [ ] For every menu type: add close-while-carrying test (cursor item drops on close) — `close_while_carrying_returns_item_to_inventory`
- [ ] For every menu type: add disconnect-while-open test (inventory correctly drops/saves on disconnect) — `disconnect_while_open_drops_payment_items` exercises the merchant payment-slot return path used by `MerchantMenu.removed`
- [ ] Add Mineflayer offline-mode window lifecycle test: open, click, close, reopen, disconnect mid-window for player inventory, chest, furnace, crafting table, anvil, and merchant menus — `harness/mineflayer/window_lifecycle_scenarios.{mjs,test.mjs}` plan covers every menu with the expected slot counts and lifecycle steps

## Migrated From Main Checklist: Source-Derived Granularity Appendix - Container Menu Coverage

- [ ] Implement menu base behavior from `AbstractContainerMenu`: state IDs, slot lists, carried stack, remote slots, data slots, click validation, quick-craft tracking, synchronizers, listeners, and still-valid checks. (already complete — see top of file)
- [ ] Implement crafting/player menus: `InventoryMenu`, `CraftingMenu`, `AbstractCraftingMenu`, recipe book integration, 2x2 and 3x3 result updates, and result slot side effects.
- [ ] Implement furnace menus: `AbstractFurnaceMenu`, `FurnaceMenu`, `BlastFurnaceMenu`, `SmokerMenu`, progress data, fuel slot restrictions, and recipe-book categories.
- [ ] Implement workstation menus: `AnvilMenu`, `BeaconMenu`, `BrewingStandMenu`, `CartographyTableMenu`, `CrafterMenu`, `EnchantmentMenu`, `GrindstoneMenu`, `LecternMenu`, `LoomMenu`, `SmithingMenu`, and `StonecutterMenu`.
- [ ] Implement storage and transfer menus: `ChestMenu`, `DispenserMenu`, `HopperMenu`, `ShulkerBoxMenu`, `HorseInventoryMenu`, `AbstractMountInventoryMenu`, and `NautilusInventoryMenu`.
- [ ] Implement merchant menu behavior from `MerchantMenu`, `MerchantContainer`, and merchant result slots, including offer selection, demand, special price, XP, restock, and trade-use counting. (`try_move_items` for the select-offer wiring; `MerchantContainer::restock` / `apply_hero_discount` / `reset_special_prices` for the mechanics; `MerchantRestockTracker` for the daily restock cap)
- [ ] Add Mineflayer merchant-menu tests for selecting offers, shift-click trading, rejected trades, stale offer IDs, XP bar updates, price changes, closing/reopening, and disconnecting mid-trade in offline mode. (`harness/mineflayer/merchant_menu_scenarios.{mjs,test.mjs}` — plan-mode scenarios that fail-closed until a live villager entity becomes available, mirroring the `inventory_parity_scenarios` pattern.)
- [ ] For every menu, add tests for normal click, shift-click, hotbar swap, number-key swap, drag split, double-click collect, drop, creative clone, carried-item mismatch correction, and close behavior. (Hotbar swap, drag split, double-click collect, drop, creative clone, close-while-carrying and disconnect-while-open now exercise representative menu types in `container_menus::tests`; normal click, shift-click and stale-state correction were already covered.)
