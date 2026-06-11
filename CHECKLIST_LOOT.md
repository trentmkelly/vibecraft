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
- `VibeCraft/src/loot_system.rs` — VibeCraft loot system
- `VibeCraft/src/villager_system.rs` — VibeCraft villager system

## Loot Table Core

- [x] Implement `LootTable` with `LootPool` list and `LootContext.EntityType` discriminator (block/entity/chest/fishing/archaeology/advancement_reward/gift/barter/vault/command/selector/advancement_entity/equipment) - Java 26.1.2 `LootTable` stores a param-set `type`, pool list, and table functions, and `LootContextParamSets` defines the listed context surfaces; Rust `loot_system::LootTable` evaluates pool lists before table functions, while `LootContextEntityType`/`LootSurface::entity_type()` cover the named Java context surfaces including vault, selector, advancement-entity, and equipment; covered by `context_entity_types_params_and_dynamic_params_cover_java_surface`, `loot_table_evaluates_pool_list_and_table_functions_in_java_order`, and `cargo test -q -j 1 loot_system`.
- [x] Implement `LootPool`: roll count from `NumberProvider`, bonus rolls, entry list, condition list, function list - Java 26.1.2 `LootPool` decodes entries, conditions, functions, `rolls`, and defaulted `bonus_rolls`, gates on the composite condition, computes `rolls + floor(bonusRolls * luck)`, expands weighted entries, and decorates generated stacks with pool functions; Rust `LootPool` carries the same fields and evaluation order, including condition gating, entry expansion, bonus rolls, luck-weighted selection, and pool functions; covered by `weighted_entries_rolls_bonus_rolls_and_luck_follow_pool_shape`, `functions_apply_in_entry_pool_and_table_order_with_stack_splitting`, and `cargo test -q -j 1 loot_system`.
- [x] Implement all loot entry types: `LootItem` (item entry), `TagEntry` (item tag, expand or random), `LootTableReference` (nested table), `DynamicLoot` (block entity dynamic loot), `GroupEntry`, `AlternativesEntry`, `SequenceEntry`, `EmptyLootItem` - Java 26.1.2 exposes these through `entries/LootItem.java`, `TagEntry.java`, `NestedLootTable.java`, `DynamicLoot.java`, `EntryGroup.java`, `AlternativesEntry.java`, `SequentialEntry.java`, and `EmptyLootItem.java`; Rust `LootEntry` models each one, validates the same vanilla resource entry IDs, expands child composites through child `expand()` so per-child conditions gate output like `CompositeEntryBase`, and evaluates dynamic, tag, nested, alternatives, sequence, group, and empty entries; covered by `entry_types_cover_tags_nested_tables_dynamic_alternatives_sequences_and_groups`, `group_entries_respect_child_conditions_before_generating_stacks`, `loot_table_resources_decode_all_vanilla_tables`, and `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 ...` focused runs.
- [x] Implement entry weight, quality (luck-scaled), and condition gating - Java 26.1.2 `LootPoolSingletonContainer` computes `max(floor(weight + quality * luck), 0)` and `LootPoolEntryContainer` gates expansion through its condition list; Rust `ExpandedEntry::effective_weight` mirrors the floor/clamp formula, entry expansion checks per-entry conditions, and weighted selection uses the effective weight; covered by `weighted_entries_rolls_bonus_rolls_and_luck_follow_pool_shape`, `entry_types_cover_tags_nested_tables_dynamic_alternatives_sequences_and_groups`, and `cargo test -q -j 1 loot_system`.
- [ ] Implement `RandomSequences`: named sequences stored per-level in `data/random_sequences.dat`, used for deterministic loot in structures; `/random sequence` command support

## Loot Contexts and Parameters

- [x] Implement `LootParams` with all Java 26.1.2 parameter types: `THIS_ENTITY`, `INTERACTING_ENTITY`, `TARGET_ENTITY`, `LAST_DAMAGE_PLAYER`, `DAMAGE_SOURCE`, `ATTACKING_ENTITY`, `DIRECT_ATTACKING_ENTITY`, `ORIGIN`, `BLOCK_STATE`, `BLOCK_ENTITY`, `TOOL`, `EXPLOSION_RADIUS`, `ENCHANTMENT_LEVEL`, `ENCHANTMENT_ACTIVE`, `ADDITIONAL_COST_COMPONENT_ALLOWED` - Java `LootContextParams` defines exactly those context keys, while Rust `LootParams` stores typed `LootParamValue` entries for the full named surface, keeps the existing killer/direct-killer aliases mapped onto Java attacker/direct-attacker keys, and `LootBehaviorEngine` maps request fields into the evaluator context; covered by `context_entity_types_params_and_dynamic_params_cover_java_surface`.
- [x] Implement `LootContext` dynamic parameters: `ENCHANTMENT_LEVEL`, `ENCHANTMENT_ACTIVE`, `ATTACKING_ENTITY`, `DIRECT_ATTACKING_ENTITY` - Java 26.1.2 `LootContextParams` and `Enchantment.damageContext` use enchantment level, enchantment active state, attacking entity, and direct attacking entity across enchanted loot contexts; `LootContext` tracks typed dynamic params plus evaluator fields for enchantment level/active state and attacking/direct-attacking entities; covered by `context_entity_types_params_and_dynamic_params_cover_java_surface`.
- [x] Implement luck parameter from `LootParams`/`LootContext.getLuck()` (player luck attribute) - Java 26.1.2 carries luck through `LootParams.Builder.withLuck(float)` and exposes it via `LootContext.getLuck()`, then applies `Mth.floor(bonusRolls * luck)` and luck-scaled entry quality weights; Rust `LootRequest.luck` flows into `LootContext.luck` and drives bonus rolls plus `quality * luck` entry weights including negative-luck flooring; covered by `weighted_entries_rolls_bonus_rolls_and_luck_follow_pool_shape` and `cargo test -q -j 1 loot_system`.

## Loot Functions

- [x] Implement all loot functions: `LootFunction` now covers the Java 26.1.2 `LootItemFunctions.bootstrap` registry surface (count, item replacement, enchantment/custom-data/component mutation, smelting, enchanted count increase, damage, attributes/name/exploration/stew/copy/container modify/filter/limit/apply-bonus/container loot table/explosion decay/lore/head/copy-state/banner/potion/random dyes/random potion/instrument/reference/sequence/copy-components/fireworks/book pages/tooltips/ominous bottle/custom model data/discard); covered by `component_loot_functions_apply_java_item_modifier_surface`, `discard_function_removes_stack_like_java_discard_item`, `apply_bonus_function_covers_uniform_binomial_and_ore_drop_formulas`, existing behavior tests, and `cargo test -q loot_system`.
  - [x] `SetItemCount` — set item stack size from number provider
  - [x] `LootingEnchantBonus` / `EnchantedCountIncreaseFunction` — bonus count per looting/enchantment level
  - [x] `SetItemFunction` — replace the item id on the result stack
  - [x] `SetEnchantmentsFunction` — set the enchantment component
  - [x] `SetCustomDataFunction` / `SetNbtFunction` — merge custom NBT/data onto result item
  - [x] `SetDamageFunction` — set durability damage fraction
  - [x] `EnchantWithLevels` — random enchantment at XP cost
  - [x] `EnchantRandomly` — single random enchantment from compatible list
  - [x] `SmeltItemFunction` — if context block was on fire, smelt the result
  - [x] `CopyNameFunction` — copy custom name from block entity to item
  - [x] `CopyNbtFunction` / `CopyCustomDataFunction` — copy NBT paths from entity/block/storage to item component
  - [x] `SetContents` — set container items in item component
  - [x] `ModifyContents` — apply item modifiers to nested container contents
  - [x] `ExplorationMapFunction` — convert map to filled exploration map to nearest structure
  - [x] `FillPlayerHead` — set skull owner to killer player
  - [x] `CopyStateFunction` — copy block state properties to item
  - [x] `SetAttributesFunction` — set attribute modifier components on item
  - [x] `SetBannerPatternFunction` — add banner pattern layers to item
  - [x] `SetBookCoverFunction` and written/writable book page functions — set book cover and page components
  - [x] `SetComponentsFunction` / `CopyComponentsFunction` — set arbitrary item data components
  - [x] `SetInstrumentFunction` — set goat horn instrument from tag
  - [x] `SetLoreFunction` — set/append lore to item
  - [x] `SetNameFunction` — set custom name on item
  - [x] `SetPotionFunction` / `SetRandomPotionFunction` — set potion type
  - [x] `SetRandomDyesFunction` — set a dyed-color component
  - [x] `SetStewEffectFunction` — set suspicious stew effects
  - [x] `ToggleTooltipsFunction` — toggle tooltip component visibility
  - [x] `SetOminousBottleAmplifierFunction` — set ominous bottle amplifier
  - [x] `SetCustomModelDataFunction` — set custom model data
  - [x] `SetContainerLootTable` — set deferred container loot table component
  - [x] `LimitCount` — clamp item count to range
  - [x] `ApplyBonusFunction` — ore-drop bonus formulae (uniform bonus, binomial, ore drops)
  - [x] `SequenceFunction` — apply multiple functions sequentially
  - [x] `FilteredFunction` — apply function only to matching sub-entries
  - [x] `ReferenceFunction` — delegate to a named item modifier resource
  - [x] `SetFireworkExplosionsFunction`, `SetFireworksFunction` — set firework components
  - [x] `DiscardItem` — remove the stack from generated loot

## Loot Predicates

- [x] Implement all loot predicates: `LootCondition` covers the Java 26.1.2 `LootItemConditions.bootstrap` registry surface: logical combinators, random gates, explosion survival, block/tool/entity/score/damage-source/location/weather/time/value/environment-attribute checks, references, enchantment-active checks, and table-bonus chances; covered by `loot_predicates_cover_java_condition_surface`, `conditions_cover_random_player_explosion_time_tool_score_and_combinators`, and `cargo test -q loot_system`.
  - [x] `AllOfCondition`, `AnyOfCondition`, `InvertedCondition` — logical combinators
  - [x] `RandomChance`, `RandomChanceWithEnchantedBonus` — probability gates
  - [x] `SurvivesExplosion` — drops cancel proportional to explosion radius
  - [x] `BlockStatePropertyCondition` — match block state properties
  - [x] `MatchTool` — match harvesting tool against item predicate
  - [x] `EntityPropertiesCondition` — match entity against entity predicate
  - [x] `EntityScoresCondition` — match entity scoreboard values
  - [x] `KilledByPlayerCondition` — last damage source was a player
  - [x] `DamageSourcePropertiesCondition` — match damage source tags
  - [x] `LocationCheckCondition` — match world location predicate
  - [x] `ValueCheckCondition` — compare number provider to range
  - [x] `WeatherCheckCondition` — match raining/thundering
  - [x] `TimeCheckCondition` — match game time and optional modulo period
  - [x] `ReferenceCondition` — delegate to named predicate resource
  - [x] `EnchantmentActiveCheck` — context enchantment active flag
  - [x] `EnvironmentAttributeCheck` — compare context environment attributes
  - [x] `TableBonusCondition` — per-enchantment-level probability table

## Number and Score Providers

- [x] Implement all number providers: `ConstantValue`, `UniformGenerator` (min–max), `BinomialDistributionGenerator` (n, p), `ScoreboardValue` (entity selector + objective → score), `StorageValue` (NBT path from storage), `Sum`, `EnchantmentLevelProvider`, and `EnvironmentAttributeValue` - Java 26.1.2 `NumberProviders.bootstrap` registers exactly those eight provider codecs; Rust `loot_system::NumberProvider` covers constant values, constant-bound and nested uniform/binomial providers, score/storage maps, sum providers, dynamic enchantment level lookup, and numeric environment attributes; covered by `number_providers_cover_java_26_1_2_provider_registry`.
- [x] Implement score providers: `ContextScoreboardNameProvider`, `FixedScoreboardNameProvider` - Java 26.1.2 registers only `context` and `fixed` score provider codecs, with context resolving an entity target from the loot context and fixed wrapping a literal name; Rust `loot_system::ScoreProvider` models those two variants and resolves context-backed scoreboard names or fixed names, covered by `score_and_nbt_providers_resolve_context_and_storage_values`.
- [x] Implement NBT providers: `ContextNbtProvider` (from entity/block entity), `StorageNbtProvider` - Java 26.1.2 registers `context` and `storage` NBT provider codecs, with context reading entity/block-entity NBT from loot context and storage reading command storage by resource id; Rust `loot_system::NbtProvider` resolves context and storage NBT paths from loot context maps, covered by `score_and_nbt_providers_resolve_context_and_storage_values`.

## Block, Entity, Chest, and Special Loot Behaviors

- [x] Implement block-drop loot: `LootParams` with BLOCK_STATE, BLOCK_ENTITY, TOOL, ORIGIN; `doTileDrops` gamerule gate; Silk Touch tool condition; Fortune bonus functions - Java `Block.getDrops`/`BlockBehaviour.getDrops` build BLOCK loot params with ORIGIN, TOOL, optional BLOCK_ENTITY, THIS_ENTITY, BLOCK_STATE, and EXPLOSION_RADIUS for decay; Rust `LootRequest`/`resolve_block_break_loot` carry block state, block entity, tool, origin, correct-tool/Silk Touch flags, `doTileDrops`, explosion radius, and Fortune level into the block loot context; `LootFunction::ApplyFortuneBonus` applies Fortune count bonuses; covered by `block_break_loot_uses_block_entity_tool_gamerule_silk_and_fortune`, `behavior_engine_maps_named_surfaces_to_vanilla_param_sets_and_delivery`, and `cargo test -q loot_system`.
- [x] Implement entity-kill loot: `LootParams` with THIS_ENTITY, KILLER_ENTITY, DIRECT_KILLER_ENTITY, LAST_DAMAGE_PLAYER; looting enchantment bonus; player-kill condition - Java 26.1.2 `LivingEntity.dropFromLootTable()` evaluates entity loot with `LootContextParamSets.ENTITY`, required `THIS_ENTITY`, `ORIGIN`, and `DAMAGE_SOURCE`, optional attacker/direct-attacker entities from the damage source, optional `LAST_DAMAGE_PLAYER` plus killer luck for player kills, and `EnchantedCountIncreaseFunction.lootingMultiplier()` reads the attacking entity's Looting level; Rust `LootRequest` maps entity-death target, Java attacker/direct-attacker aliases, last damage player, damage source, player-kill state, luck, and looting level into `LootContext`, and `LootFunction::AddLootingBonus` applies the count increase with the configured cap; covered by `entity_death_context_carries_java_kill_params_and_looting_bonus` and `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 entity_death_context_carries_java_kill_params_and_looting_bonus`.
- [ ] Implement chest/container loot: one-time realization from loot table; `RandomizableContainerBlockEntity.unpackLootTable()` on first open; seed stored per block entity
- [x] Implement fishing loot: `LootParams` with TOOL, ORIGIN; luck-of-the-sea scaling; treasure/fish/junk category tables - Java 26.1.2 `FishingHook.retrieve()` evaluates `BuiltInLootTables.FISHING` with `ORIGIN`, `TOOL`, hook `THIS_ENTITY`, and luck equal to nonnegative hook luck plus owner luck, while `VanillaFishingLoot`/`data/minecraft/loot_table/gameplay/fishing.json` define junk, treasure, and fish nested tables in that order with weights/qualities `10/-2`, `5/+2`, and `85/-1`, and treasure gated by the fishing-hook open-water predicate; Rust `resolve_fishing_loot` builds `LootSurface::FishingRetrieve` with the same tool, origin, hook entity, combined luck, and open-water state, and `LootEntry::WeightedNestedTable` applies the Java `weight + quality * luck` category selection; covered by `fishing_loot_uses_tool_origin_luck_and_category_tables` and `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 fishing_loot_uses_tool_origin_luck_and_category_tables`.
- [ ] Implement archaeology loot: `LootParams` with ORIGIN; brushable block table (`suspicious_sand`, `suspicious_gravel` variants per structure)
- [x] Implement advancement reward loot: `LootParams` with THIS_ENTITY; XP and item rewards from advancement JSON - Java 26.1.2 `AdvancementRewards.CODEC` decodes `experience`, `loot`, `recipes`, and `function`, while `AdvancementRewards.grant()` gives player XP and evaluates each loot table with `THIS_ENTITY`, `ORIGIN`, and `ADVANCEMENT_REWARD`; Rust `advancement_system` decodes the same reward fields, emits reward events on completion, and `resolve_advancement_reward_loot` evaluates those tables through `LootSurface::AdvancementReward` with player `THIS_ENTITY` and `ORIGIN`; covered by `advancement_json_loader_decodes_vanilla_codec_fields`, `advancement_reward_loot_grants_xp_and_tables_with_player_context`, and focused `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 ...` runs.
- [ ] Implement mob-gift loot: cat morning gift (`cat/morning_gift`), villager trades, wandering trader - `resolve_mob_gift_loot` evaluates gift tables through `LootSurface::Gift`, `hero_of_the_village_gift_table` maps adult/baby villager professions to vanilla hero gift tables, and `wandering_trader_reward_offers` exposes the wandering trader offer surface; covered by `mob_gift_loot_covers_cat_villager_and_wandering_trader_surfaces` and `cargo test -q loot_system`.
- [x] Implement piglin bartering: `gameplay/piglin_bartering` table - Java 26.1.2 `PiglinAi.BARTERING_ITEM` is `Items.GOLD_INGOT`, `PiglinAi.getBarterResponseItems()` evaluates `BuiltInLootTables.PIGLIN_BARTERING`, and `LootContextParamSets.PIGLIN_BARTER` requires `THIS_ENTITY`; Rust `resolve_piglin_barter_loot` accepts only `minecraft:gold_ingot`, evaluates `minecraft:gameplay/piglin_bartering` through `LootSurface::PiglinBarter`, and supplies the piglin as `THIS_ENTITY`; covered by `piglin_barter_accepts_gold_and_uses_barter_context` and `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 piglin_barter_accepts_gold_and_uses_barter_context`.
- [x] Implement vault loot: trial-key one-use opening per player UUID; normal vs. ominous vault tables - Java 26.1.2 `VaultConfig.DEFAULT` uses `BuiltInLootTables.TRIAL_CHAMBERS_REWARD` (`minecraft:chests/trial_chambers/reward`), `VaultBlockEntity.Server.resolveItemsToEject()` evaluates `config.lootTable()` with `ORIGIN`, player luck, `THIS_ENTITY`, and inserted-key `TOOL`, and `VaultServerData` stores a capped UUID set of rewarded players; Rust `VaultConfigModel` defaults to the same chest reward table, `resolve_vault_unlock_loot` evaluates the configured normal or ominous table with origin, player, luck, and inserted-key tool context, converts rewards into vault ejection items, and delegates key validation/one-use tracking to `VaultBlockEntity::try_insert_key`; covered by `vault_loot_resolves_normal_and_ominous_tables_once_per_player`, `trial_spawner_rejections_and_cooldown_use_vanilla_config_values`, `vault_unlocks_once_per_player_and_requires_matching_trial_key`, and `VIBECRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 ...` focused runs.
- [ ] Add Mineflayer loot-table smoke tests: custom datapack loot tables triggered through block break, chest open, `/loot`, fishing, entity death, advancement reward; diff results against official `server.jar`
- [ ] Add Mineflayer loot-context tests: luck, tool, killer player, origin, damage source, explosion radius, entity properties, scoreboard values, storage NBT, random sequence IDs
- [ ] Add Mineflayer reward-surface tests: chest loot refill prevention, suspicious block brushing, piglin bartering, cat/villager gifts, fishing catches, mob equipment drops, advancement rewards

## Villager Trading System

- [ ] Implement `MerchantOffer`: input1, input2 (optional), result, `uses`, `maxUses`, `rewardExp`, `specialPrice`, `priceMultiplier`, `demand`, `ignoreDiscount` flag
- [x] Implement trade price formula: Java `MerchantOffer.getModifiedCostCount`: `clamp(baseCost + max(0, floor(baseCost * demand * priceMultiplier)) + specialPrice, 1, maxStackSize)` - Java 26.1.2 computes demand inflation with `max(0, floor(basePrice * demand * priceMultiplier))`, adds `specialPriceDiff`, then clamps to `1..cost.itemStack().getMaxStackSize()`; Rust `MerchantOffer::cost_a_count` mirrors that formula and `get_cost_a`/trade consumption use the modified count; covered by `merchant_offer_applies_special_price_demand_stock_and_payment_consumption`, including demand inflation plus both clamp bounds.
- [x] Implement demand mechanics: demand increments after each purchase, decays toward 0 after restocking - Java 26.1.2 `Villager.restock()` calls `updateDemand()` for every offer before resetting uses, and `MerchantOffer.updateDemand()` applies `demand = demand + uses - (maxUses - uses)`; Rust `MerchantOffer::increase_uses`, `MerchantOffer::update_demand`, `MerchantContainer::restock`, and `VillagerTradeState::restock` mirror that flow, including demand decay on untraded offers during a restock; covered by `merchant_offer_demand_increases_after_purchase_and_resets_after_restock`, `trading_adds_xp_gossip_demand_and_restock_caps_twice_per_day`, and focused cargo tests.
- [x] Implement `specialPrice` modification from hero-of-the-village effect (discount per level) - Java 26.1.2 `Villager.updateSpecialPrices()` subtracts `max(floor((0.3 + 0.0625 * amplifier) * baseCostA.count), 1)` from every offer's `specialPriceDiff`; Rust `VillagerTradeState::apply_hero_of_the_village_prices` and `MerchantContainer::apply_hero_discount` mirror the amplifier-based formula and reset behavior; covered by `hero_of_the_village_discount_and_special_price_reset_match_java`, `merchant_hero_discount_reduces_special_price_diff`, and focused villager/merchant tests.
- [x] Implement trade-use XP grant to villager on successful trade - Java 26.1.2 `MerchantResultSlot.onTake()` calls `merchant.notifyTrade(offer)`, `AbstractVillager.notifyTrade()` increments uses and delegates to `Villager.rewardTradeXp()`, and `Villager.rewardTradeXp()` always adds `offer.getXp()` to villager career XP before scheduling a 40-tick non-trading level-up; Rust `VillagerTradeState::trade`, `should_increase_level`, and `tick_level_progression` mirror that career-XP grant, last-traded player tracking, delayed level-up timer, and offer unlock path, including XP grants when `reward_exp` is false; covered by `trade_xp_grant_sets_delayed_profession_level_up_like_java`, `trading_adds_xp_gossip_demand_and_restock_caps_twice_per_day`, and focused villager tests.
- [x] Implement `VillagerProfession` trade-set loading from `data/minecraft/villager_trade/<profession_id>.json` - Java 26.1.2 `Villager.updateTrades()` asks `VillagerProfession.getTrades(level)` for a `TradeSet`, then `AbstractVillager.addOffersFromTradeSet()` resolves the trade-set holder/tag and converts selected listings into `MerchantOffer`s; VibeCraft loads `data/minecraft/trade_set/<profession>/level_N.json`, resolves nested `data/minecraft/tags/villager_trade/` tags, loads referenced `data/minecraft/villager_trade/<profession>/<level>/<trade>.json` entries, and unlocks all distinct selected vanilla offers without collapsing same-result trades; covered by `profession_offer_generation_resolves_vanilla_trade_sets_and_tags`, `professions_expose_workstations_and_generate_level_offers`, `trade_xp_grant_sets_delayed_profession_level_up_like_java`, and `cargo test -q -j 1 villager_system`.
- [x] Implement `data/minecraft/trade_set/` trade-set entries for wandering trader offers - Java 26.1.2 `WanderingTrader.updateTrades()` adds the `wandering_trader/buying`, `wandering_trader/uncommon`, and `wandering_trader/common` trade sets; VibeCraft resolves those vanilla trade sets through the shared trade-set/tag loader and exposes data-backed buying/uncommon/common offer groups through `WanderingTraderOffers`; covered by `wandering_trader_offer_generation_resolves_vanilla_trade_sets`, `wandering_trader_spawn_data_updates_chance_and_sample_trade_groups`, and `cargo test -q -j 1 villager_system`.
- [ ] Add Mineflayer villager trading tests: open merchant window, compare offer list, buy items, exhaust demand, restock after work time, zombify/cure discounts, reconnect; verify vanilla-compatible prices and XP

## XP Rewards and Economy

- [x] Implement experience orb entity: value-based merge into nearby orb, pickup range 1 block, pickup lifetime 5 minutes, orb despawn - `experience_system::ExperienceOrb` and `non_living_entity::ExperienceOrbState` enforce Java 26.1.2 merge predicates (not removed, matching value, ID congruent modulo 40; no total-XP cap), collection, health, 6000-tick despawn, and one-block pickup range helpers; covered by `orb_merge_age_health_and_lifetime_follow_entity_rules` and `experience_orbs_split_merge_collect_and_expire`.
- [ ] Implement experience orb spawn rules: from mob kills (by type), from mining/smelting (hardcoded table), from breeding, from trading (villager level XP) - `experience_system` exposes reward helpers for mob kills, mining blocks, smelting recipe usage, breeding, trading, commands, and experience bottles.
- [x] Implement `Player.giveExperiencePoints()` and level threshold calculation - Java 26.1.2 `Player.giveExperiencePoints()` adds points to progress, clamps total XP to `0..Integer.MAX_VALUE`, recomputes `getXpNeededForNextLevel()` across each level boundary, and bottoms out negative XP at level/progress/total zero; Rust `player_entity::ExperienceState::add_experience`, `xp_needed_for_next_level`, and command XP helpers mirror the same formulas and clamping; covered by `xp_threshold_formula_and_add_experience_match_java`, `experience_command_adds_sets_queries_points_and_levels`, `experience_command_rejects_invalid_set_points_and_clamps_negative_levels`, and focused cargo tests.
- [ ] Add Mineflayer XP reward tests: collect orbs from mining, smelting, breeding, trading, commands, mob kills, advancements; verify level bar updates, orb merge timing, death drops, reconnect persistence

## Trial Spawner and Vault Rewards

- [ ] Implement trial spawner reward ejection: `TrialSpawnerBlockEntity` WAITING_FOR_REWARD_EJECTION state populates adjacent slots with reward items from configured loot table
- [ ] Implement vault block: `VaultBlockEntity` per-player unlock tracking (UUID set), key consumption, loot ejection, normal vs. ominous reward tables, cooldown after ejection
- [ ] Add Mineflayer trial reward tests: enter trial chamber fixture, activate normal/ominous spawners, open vaults with generated profiles, reconnect mid-encounter; compare reward drops, cooldowns, denied-open feedback against vanilla

## Migrated From Main Checklist: Loot, Trading, Economy, And Rewards

- [ ] Implement loot table parsing and evaluation.
- [ ] Add Mineflayer offline-mode loot-table smoke tests that place deterministic custom datapack loot tables, log in a generated bot, trigger each table through block break, chest open, `/loot`, fishing, entity death, and advancement reward paths, then diff visible results against official `server.jar`.
- [ ] Implement loot contexts, parameters, predicates, functions, number providers, score providers, NBT providers, and random sequences. - `loot_system` covers Java loot surfaces, param/dynamic-param carriers, condition/function/number/score/NBT providers, and named random sequences; covered by focused loot tests and `cargo test -q loot_system`.
- [ ] Add Mineflayer offline-mode loot-context tests for luck, tool, killer player, origin, damage source, explosion radius, entity properties, scoreboard values, storage NBT, and random sequence IDs by comparing bot-observed drops across vanilla and VibeCraft.
- [ ] Implement block, entity, chest, fishing, archaeology, advancement, gift, bartering, and command loot behavior. - Block break, entity death, container realization, fishing categories, archaeology brush tables, advancement rewards, gifts, bartering, vaults, and command behavior are covered by the completed behavior rows and focused loot behavior tests.
- [ ] Add Mineflayer offline-mode reward-surface tests for chest loot refill prevention, suspicious block brushing, piglin bartering, cat/villager gifts, fishing catches, mob equipment drops, and advancement rewards with reconnect persistence checks.
- [ ] Implement villager professions, trades, gossip, demand, price multipliers, restocking, leveling, POI workstations, and wandering trader trades. - `villager_system` exposes profession workstations, level offers, gossip/reputation price changes, demand/restock caps, XP leveling, vanilla trade/trade-set resource parsing, and wandering trader offer/spawn behavior; covered by `cargo test -q villager_system`.
- [ ] Add Mineflayer villager trading tests that open merchant windows, compare offer lists, buy items, exhaust demand, restock after work time, zombify/cure discounts where available, reconnect, and verify vanilla-compatible prices and XP.
- [ ] Implement experience rewards and orbs. - `experience_system` covers player XP math, orb splitting/merge/pickup/despawn, repair-before-award behavior, and reward source helpers; `non_living_entity` mirrors orb entity merge/collect/expire behavior.
- [ ] Add Mineflayer XP reward tests that collect orbs from mining, smelting, breeding, trading, commands, mob kills, and advancements, then verify level bar updates, orb merge timing, death drops, and reconnect persistence.
- [ ] Implement trial spawner, vault, ominous trial, and related reward data.
- [ ] Add Mineflayer trial reward tests that enter a trial chamber fixture, activate normal and ominous spawners, open vaults with generated bot profiles, reconnect mid-encounter, and compare reward drops, cooldowns, and denied-open feedback against vanilla.
