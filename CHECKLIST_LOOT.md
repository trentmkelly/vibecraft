# Loot, Trading, and Economy Checklist

## Relevant Java Source Files

- `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/` — loot table system
- `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/LootTable.java` — loot table entry point
- `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/LootPool.java` — pool within a loot table
- `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/LootContext.java` — evaluation context
- `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/LootParams.java` — parameter values
- `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/entries/` — entry types
- `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/` — loot functions
- `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/` — loot predicates
- `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/` — number/score/NBT providers
- `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/RandomSequences.java` — named random sequences
- `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/BuiltInLootTables.java` — built-in table registry
- `decompiled-server-26.1.2/net/minecraft/world/item/trading/MerchantOffer.java` — villager trade offer
- `decompiled-server-26.1.2/net/minecraft/world/item/trading/MerchantOffers.java` — offers list
- `decompiled-server-26.1.2/net/minecraft/world/entity/npc/VillagerProfession.java` — villager professions
- `decompiled-server-26.1.2/data/minecraft/loot_table/` — all vanilla loot table JSON files
- `decompiled-server-26.1.2/data/minecraft/villager_trade/` — villager trade set JSON files
- `RustCraft/src/loot_system.rs` — RustCraft loot system
- `RustCraft/src/villager_system.rs` — RustCraft villager system

## Loot Table Core

- [x] Implement `LootTable` with `LootPool` list and `LootContext.EntityType` discriminator (block/entity/chest/fishing/archaeology/advancement_reward/gift/barter/vault/command/selector/advancement_entity/equipment) - `loot_system::LootTable` evaluates pool lists and table functions, while `LootContextEntityType`/`LootSurface::entity_type()` cover the named Java context surfaces including vault, selector, advancement-entity, and equipment.
- [x] Implement `LootPool`: roll count from `NumberProvider`, bonus rolls, entry list, condition list, function list
- [x] Implement all loot entry types: `LootItem` (item entry), `TagEntry` (item tag, expand or random), `LootTableReference` (nested table), `DynamicLoot` (block entity dynamic loot), `GroupEntry`, `AlternativesEntry`, `SequenceEntry`, `EmptyLootItem`
- [x] Implement entry weight, quality (luck-scaled), and condition gating
- [x] Implement `RandomSequences`: named sequences stored per-level in `data/random_sequences.dat`, used for deterministic loot in structures; `/random sequence` command support

## Loot Contexts and Parameters

- [x] Implement `LootParams` with all parameter types: `BLOCK_STATE`, `BLOCK_ENTITY`, `ORIGIN`, `TOOL`, `THIS_ENTITY`, `LAST_DAMAGE_PLAYER`, `KILLER_ENTITY`, `DIRECT_KILLER_ENTITY`, `EXPLOSION_RADIUS`, `DAMAGE_SOURCE` - `loot_system::LootParams` stores typed `LootParamValue` entries for the full named parameter surface and `LootBehaviorEngine` maps request fields into the evaluator context.
- [x] Implement `LootContext` dynamic parameters: `ENCHANTMENT_LEVEL`, `ENCHANTMENT_ACTIVE`, `ATTACKING_ENTITY` - `LootContext` now tracks typed dynamic params plus evaluator fields for enchantment level/active state and attacking entity; covered by `context_entity_types_params_and_dynamic_params_cover_java_surface`.
- [x] Implement luck parameter from `LootContext.LUCK` (player luck attribute)

## Loot Functions

- [ ] Implement all loot functions:
  - [ ] `SetItemCount` — set item stack size from number provider
  - [ ] `LootingEnchantBonus` — bonus count per looting level
  - [ ] `SetDamageFunction` — set durability damage fraction
  - [ ] `SetNbtFunction` — merge NBT onto result item
  - [ ] `EnchantWithLevels` — random enchantment at XP cost
  - [ ] `EnchantRandomly` — single random enchantment from compatible list
  - [ ] `SmeltItemFunction` — if context block was on fire, smelt the result
  - [ ] `CopyNameFunction` — copy custom name from block entity to item
  - [ ] `CopyNbtFunction` — copy NBT paths from entity/block/storage to item component
  - [ ] `SetContents` — set container items in item component
  - [ ] `ExplorationMapFunction` — convert map to filled exploration map to nearest structure
  - [ ] `FillPlayerHead` — set skull owner to killer player
  - [ ] `CopyStateFunction` — copy block state properties to item
  - [ ] `SetAttributesFunction` — set attribute modifier components on item
  - [ ] `SetBannerPatternFunction` — add banner pattern layers to item
  - [ ] `SetBookContentsFunction` — set written book pages/title/author
  - [ ] `SetComponentsFunction` — set arbitrary item data components
  - [ ] `SetInstrumentFunction` — set goat horn instrument from tag
  - [ ] `SetLoreFunction` — set/append lore to item
  - [ ] `SetNameFunction` — set custom name on item
  - [ ] `SetPotionFunction` — set potion type
  - [ ] `SetStewEffectFunction` — set suspicious stew effects
  - [ ] `SetWrittenBookPagesFunction` — set writable/written book pages
  - [ ] `ToggleTooltipsFunction` — toggle tooltip component visibility
  - [ ] `LimitCount` — clamp item count to range
  - [ ] `ApplyBonusFunction` — ore-drop bonus formulae (uniform bonus, binomial, ore drops)
  - [ ] `SequenceFunction` — apply multiple functions sequentially
  - [ ] `FilteredFunction` — apply function only to matching sub-entries
  - [ ] `ReferenceFunction` — delegate to a named item modifier resource
  - [ ] `SetFireworkExplosionsFunction`, `SetFireworksFunction` — set firework components

## Loot Predicates

- [ ] Implement all loot predicates:
  - [ ] `AllOfCondition`, `AnyOfCondition`, `InvertedCondition` — logical combinators
  - [ ] `RandomChance`, `RandomChanceWithEnchantedBonus` — probability gates
  - [ ] `SurvivesExplosion` — drops cancel proportional to explosion radius
  - [ ] `BlockStatePropertyCondition` — match block state properties
  - [ ] `MatchTool` — match harvesting tool against item predicate
  - [ ] `EntityPropertiesCondition` — match entity against entity predicate
  - [ ] `EntityScoresCondition` — match entity scoreboard values
  - [ ] `KilledByPlayerCondition` — last damage source was a player
  - [ ] `LootingRandomChance` — scaled chance per looting level
  - [ ] `DamageSourcePropertiesCondition` — match damage source tags
  - [ ] `LocationCheckCondition` — match world location predicate
  - [ ] `ValueCheckCondition` — compare number provider to range
  - [ ] `WeatherCheckCondition` — match raining/thundering
  - [ ] `TimeCheckCondition` — match game time modulo period
  - [ ] `ReferenceCondition` — delegate to named predicate resource
  - [ ] `EnchantmentActiveCheck` — context enchantment active flag
  - [ ] `TableBonusCondition` — per-enchantment-level probability table

## Number and Score Providers

- [x] Implement all number providers: `ConstantValue`, `UniformGenerator` (min–max), `BinomialDistributionGenerator` (n, p), `ScoreboardValue` (entity selector + objective → score), `StorageValue` (NBT path from storage), `EnchantmentLevelProvider` - `loot_system::NumberProvider` covers constant, uniform, binomial, score, storage, and enchantment-level values, including score/storage maps and dynamic enchantment level lookup.
- [x] Implement score providers: `ContextScoreboardNameProvider`, `FixedScoreboardNameProvider` - `loot_system::ScoreProvider` resolves context-backed scoreboard names and fixed names; covered by `score_and_nbt_providers_resolve_context_and_storage_values` and `cargo test -q loot_system`.
- [x] Implement NBT providers: `ContextNbtProvider` (from entity/block entity), `StorageNbtProvider` - `loot_system::NbtProvider` resolves context and storage NBT paths from loot context maps; covered by `score_and_nbt_providers_resolve_context_and_storage_values` and `cargo test -q loot_system`.

## Block, Entity, Chest, and Special Loot Behaviors

- [x] Implement block-drop loot: `LootParams` with BLOCK_STATE, BLOCK_ENTITY, TOOL, ORIGIN; `doTileDrops` gamerule gate; Silk Touch tool condition; Fortune bonus functions - `LootRequest`/`resolve_block_break_loot` carry block state, block entity, tool, origin, correct-tool/Silk Touch flags, `doTileDrops`, and Fortune level into the block loot context; `LootFunction::ApplyFortuneBonus` applies Fortune count bonuses; covered by `block_break_loot_uses_block_entity_tool_gamerule_silk_and_fortune` and `cargo test -q loot_system`.
- [x] Implement entity-kill loot: `LootParams` with THIS_ENTITY, KILLER_ENTITY, DIRECT_KILLER_ENTITY, LAST_DAMAGE_PLAYER; looting enchantment bonus; player-kill condition - `LootRequest` maps entity-death target, killer, direct killer, last damage player, damage source, player-kill state, and looting level into `LootContext`; `LootFunction::AddLootingBonus` applies looting count bonuses; covered by `entity_death_context_carries_java_kill_params_and_looting_bonus` and `cargo test -q loot_system`.
- [x] Implement chest/container loot: one-time realization from loot table; `RandomizableContainerBlockEntity.unpackLootTable()` on first open; seed stored per block entity
- [x] Implement fishing loot: `LootParams` with TOOL, ORIGIN; luck-of-the-sea scaling; treasure/fish/junk category tables - `resolve_fishing_loot` builds the fishing surface with `TOOL`, `ORIGIN`, hook `THIS_ENTITY`, hook/player luck, and open-water state; `LootEntry::WeightedNestedTable` models the vanilla fishing category table weights/quality luck scaling for junk, fish, and treasure; covered by `fishing_loot_uses_tool_origin_luck_and_category_tables` and `cargo test -q loot_system`.
- [x] Implement archaeology loot: `LootParams` with ORIGIN; brushable block table (`suspicious_sand`, `suspicious_gravel` variants per structure)
- [x] Implement advancement reward loot: `LootParams` with THIS_ENTITY; XP and item rewards from advancement JSON - `advancement_system` emits XP and loot-table reward events from advancement JSON/progress completion, and `resolve_advancement_reward_loot` evaluates those tables through `LootSurface::AdvancementReward` with player `THIS_ENTITY` and `ORIGIN`; covered by `advancement_reward_loot_grants_xp_and_tables_with_player_context` and `cargo test -q loot_system`.
- [ ] Implement mob-gift loot: cat morning gift (`cat/morning_gift`), villager trades, wandering trader
- [x] Implement piglin bartering: `gameplay/piglin_bartering` table - `resolve_piglin_barter_loot` accepts only `minecraft:gold_ingot`, evaluates `minecraft:gameplay/piglin_bartering` through `LootSurface::PiglinBarter`, and supplies the piglin as `THIS_ENTITY`; covered by `piglin_barter_accepts_gold_and_uses_barter_context` and `cargo test -q loot_system`.
- [ ] Implement vault loot: trial-key one-use opening per player UUID; normal vs. ominous vault tables
- [x] Add Mineflayer loot-table smoke tests: custom datapack loot tables triggered through block break, chest open, `/loot`, fishing, entity death, advancement reward; diff results against official `server.jar`
- [x] Add Mineflayer loot-context tests: luck, tool, killer player, origin, damage source, explosion radius, entity properties, scoreboard values, storage NBT, random sequence IDs
- [x] Add Mineflayer reward-surface tests: chest loot refill prevention, suspicious block brushing, piglin bartering, cat/villager gifts, fishing catches, mob equipment drops, advancement rewards

## Villager Trading System

- [x] Implement `MerchantOffer`: input1, input2 (optional), result, `uses`, `maxUses`, `rewardExp`, `specialPrice`, `priceMultiplier`, `demand`, `ignoreDiscount` flag
- [x] Implement trade price formula: Java `MerchantOffer.getModifiedCostCount`: `clamp(baseCost + max(0, floor(baseCost * demand * priceMultiplier)) + specialPrice, 1, maxStackSize)`
- [x] Implement demand mechanics: demand increments after each purchase, decays toward 0 after restocking
- [x] Implement `specialPrice` modification from hero-of-the-village effect (discount per level)
- [x] Implement trade-use XP grant to villager on successful trade
- [x] Implement `VillagerProfession` trade-set loading from `data/minecraft/villager_trade/<profession_id>.json`
- [x] Implement `data/minecraft/trade_set/` trade-set entries for wandering trader offers
- [x] Add Mineflayer villager trading tests: open merchant window, compare offer list, buy items, exhaust demand, restock after work time, zombify/cure discounts, reconnect; verify vanilla-compatible prices and XP

## XP Rewards and Economy

- [x] Implement experience orb entity: value-based merge into nearby orb (max orb merge = total XP ≤ 10 per group), pickup range 1 block, pickup lifetime 5 minutes, orb despawn - `experience_system::ExperienceOrb` and `non_living_entity::ExperienceOrbState` enforce merge grouping, collection, health, 6000-tick despawn, and one-block pickup range helpers; covered by focused XP/orb tests.
- [x] Implement experience orb spawn rules: from mob kills (by type), from mining/smelting (hardcoded table), from breeding, from trading (villager level XP) - `experience_system` exposes reward helpers for mob kills, mining blocks, smelting recipe usage, breeding, trading, commands, and experience bottles.
- [x] Implement `Player.giveExperiencePoints()` and level threshold calculation
- [x] Add Mineflayer XP reward tests: collect orbs from mining, smelting, breeding, trading, commands, mob kills, advancements; verify level bar updates, orb merge timing, death drops, reconnect persistence

## Trial Spawner and Vault Rewards

- [x] Implement trial spawner reward ejection: `TrialSpawnerBlockEntity` WAITING_FOR_REWARD_EJECTION state populates adjacent slots with reward items from configured loot table
- [x] Implement vault block: `VaultBlockEntity` per-player unlock tracking (UUID set), key consumption, loot ejection, normal vs. ominous reward tables, cooldown after ejection
- [x] Add Mineflayer trial reward tests: enter trial chamber fixture, activate normal/ominous spawners, open vaults with generated profiles, reconnect mid-encounter; compare reward drops, cooldowns, denied-open feedback against vanilla

## Migrated From Main Checklist: Loot, Trading, Economy, And Rewards

- [x] Implement loot table parsing and evaluation.
- [x] Add Mineflayer offline-mode loot-table smoke tests that place deterministic custom datapack loot tables, log in a generated bot, trigger each table through block break, chest open, `/loot`, fishing, entity death, and advancement reward paths, then diff visible results against official `server.jar`.
- [ ] Implement loot contexts, parameters, predicates, functions, number providers, score providers, NBT providers, and random sequences.
- [x] Add Mineflayer offline-mode loot-context tests for luck, tool, killer player, origin, damage source, explosion radius, entity properties, scoreboard values, storage NBT, and random sequence IDs by comparing bot-observed drops across vanilla and RustCraft.
- [ ] Implement block, entity, chest, fishing, archaeology, advancement, gift, bartering, and command loot behavior.
- [x] Add Mineflayer offline-mode reward-surface tests for chest loot refill prevention, suspicious block brushing, piglin bartering, cat/villager gifts, fishing catches, mob equipment drops, and advancement rewards with reconnect persistence checks.
- [x] Implement villager professions, trades, gossip, demand, price multipliers, restocking, leveling, POI workstations, and wandering trader trades. - `villager_system` exposes profession workstations, level offers, gossip/reputation price changes, demand/restock caps, XP leveling, vanilla trade/trade-set resource parsing, and wandering trader offer/spawn behavior; covered by `cargo test -q villager_system`.
- [x] Add Mineflayer villager trading tests that open merchant windows, compare offer lists, buy items, exhaust demand, restock after work time, zombify/cure discounts where available, reconnect, and verify vanilla-compatible prices and XP.
- [x] Implement experience rewards and orbs. - `experience_system` covers player XP math, orb splitting/merge/pickup/despawn, repair-before-award behavior, and reward source helpers; `non_living_entity` mirrors orb entity merge/collect/expire behavior.
- [x] Add Mineflayer XP reward tests that collect orbs from mining, smelting, breeding, trading, commands, mob kills, and advancements, then verify level bar updates, orb merge timing, death drops, and reconnect persistence.
- [x] Implement trial spawner, vault, ominous trial, and related reward data.
- [x] Add Mineflayer trial reward tests that enter a trial chamber fixture, activate normal and ominous spawners, open vaults with generated bot profiles, reconnect mid-encounter, and compare reward drops, cooldowns, and denied-open feedback against vanilla.
