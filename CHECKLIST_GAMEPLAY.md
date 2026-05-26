# Gameplay Mechanics Checklist

## Relevant Java Source Files

- `decompiled-server-26.1.2/net/minecraft/world/level/block/` — block interaction logic
- `decompiled-server-26.1.2/net/minecraft/server/level/ServerPlayerGameMode.java` — block breaking, placing
- `decompiled-server-26.1.2/net/minecraft/world/item/context/UseOnContext.java` — item use context
- `decompiled-server-26.1.2/net/minecraft/world/effect/MobEffect.java` — status effect base
- `decompiled-server-26.1.2/net/minecraft/world/effect/MobEffects.java` — effect registry
- `decompiled-server-26.1.2/net/minecraft/world/effect/MobEffectInstance.java` — applied effect instance
- `decompiled-server-26.1.2/net/minecraft/world/damagesource/DamageSource.java` — damage source
- `decompiled-server-26.1.2/net/minecraft/world/damagesource/CombatTracker.java` — death message tracking
- `decompiled-server-26.1.2/net/minecraft/world/entity/LivingEntity.java` — damage/combat/effects
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/WeatherType.java` — weather state
- `decompiled-server-26.1.2/net/minecraft/server/level/ServerLevel.java` — level tick (weather, time, sleep)
- `decompiled-server-26.1.2/net/minecraft/world/level/border/WorldBorder.java` — world border
- `decompiled-server-26.1.2/net/minecraft/world/level/border/WorldBorderPhase.java` — border lerp
- `decompiled-server-26.1.2/net/minecraft/world/entity/player/SleepStatus.java` — sleep counting
- `decompiled-server-26.1.2/net/minecraft/world/phys/BlockHitResult.java` — block hit context
- `RustCraft/src/combat_damage.rs` — RustCraft combat
- `RustCraft/src/status_effect.rs` — RustCraft status effects
- `RustCraft/src/weather.rs` — RustCraft weather
- `RustCraft/src/world_border.rs` — RustCraft world border
- `RustCraft/src/world_time.rs` — RustCraft time
- `RustCraft/src/block_behavior.rs` — RustCraft block behavior

## Block Placement and Breaking

- [ ] Add Mineflayer block place/break tests for offline-mode survival and creative bots: reach distance enforcement (5.0 survival, creative extends same server-side), denied placement at spawn-protection, placement orientation by face hit, drops from breaking, visible block-update acknowledgment packets
- [ ] Add Mineflayer block interaction parity tests: right-click use, sneak-use bypass (sneaking + use item), client prediction rollback (server rejects placement → sends corrective update), block entity update tags sent after right-click, neighbor-shape updates after offline-mode placement
- [ ] Add Mineflayer offline-mode block-entity interaction tests: signs (edit/wax/dye), chests (open/loot), furnaces (insert/extract), lecterns (page turn), bells (ring broadcast), note blocks (pitch), campfires (cook), cauldrons (fill/empty), spawners (spawn egg insert), brushable blocks (brush progress)
- [ ] Add Mineflayer offline-mode block-drop tests: break representative blocks with bare hand (no drop), correct tool (drop), Silk Touch (block itself drops), Fortune level 1/2/3 (bonus count), explosion (0% per-block survival), `doTileDrops=false` gamerule (no drops); compare inventory pickups and world item entities against official `server.jar`
- [ ] Implement `ServerPlayerGameMode.handleBlockBreakAction()`: start-dig / abort-dig / stop-dig sequence, break progress calculation per tool efficiency and haste/mining-fatigue, instant-break in creative, adventure-mode CanDestroy tag restriction
- [ ] Implement server-side block-placement validation: block-reach distance check, `canSurvive()` check on target position, entity collision check (cannot place inside entities), spawn-protection check
- [ ] Implement `Block.use()` / `Block.attack()` dispatch: route right-click to block use handler first, then item use handler, respecting `InteractionResult` (SUCCESS/CONSUME/FAIL/PASS)
- [x] Implement neighbor notification cascade: when a block changes, call `updateIndirectNeighbourShapes()` and `onNeighborChanged()` for all 6 adjacent blocks and their adjacent blocks (shape-update chain) — `block_update::NeighborUpdateQueue::update_shape_cascade()` now emits first-layer neighbor notifications plus the second-layer shape-update chain around each neighbor, respecting the configured chained-update limit; covered by `cargo test -q block_update`.
- [ ] Add parity test: break-speed calculation for diamond pickaxe on stone vs. dirt vs. obsidian matches vanilla ticks
- [x] Add parity test: neighbor update cascade when placing/breaking redstone wire propagates signal changes to all affected comparators and repeaters — `vanilla_trace_redstone_wire_neighbor_cascade_reaches_comparators_and_repeaters` verifies both redstone-wire placement and break cascade through second-layer neighbor notifications to comparator and repeater positions; covered by `cargo test -q redstone_wire_neighbor_cascade`.

## Combat and Damage

- [ ] Add Mineflayer combat/damage tests: melee attack (hit animation, damage value, knockback), projectile (arrow damage falloff with range), fall damage (formula: `max(0, height - 3) * 1.5`), fire damage (1/tick), drowning damage (2/tick when air = 0), void damage (4/tick below world bottom), shield blocking (negate projectile + reduce melee), armor mitigation (damage factor formula), invulnerability frames (0.5 s), vanilla-compatible damage/death messages
- [x] Implement `LivingEntity.hurt()`: invulnerability frame check, absorb through absorption attribute, armor/enchantment/effect protection calculation, knockback application, death check — `LivingEntityState::hurt()` now applies cooldown delta rules, armor plus enchantment/effect protection, absorption, hurt/death timers, and death animation; `hurt_with_knockback()` applies knockback only when damage lands; covered by `cargo test -q living_entity`.
- [x] Implement armor protection formula: `toughness = 2 + armorToughness / 4; effectiveArmor = clamp(armor - damage / toughness, armor * 0.2, 20); result = damage * (1 - effectiveArmor / 25)` (vanilla `CombatRules.getDamageAfterAbsorb`) — `combat_damage::damage_after_armor` matches Java, `damage_after_magic_absorb` matches `getDamageAfterMagicAbsorb`; verified by reading Java `CombatRules.java` and comparing numeric outputs for iron/diamond armor
- [ ] Implement `CombatTracker`: track last-damage source and killer for death message generation, `getDeathMessage()` component building using `DamageType.deathMessageType()`
- [ ] Implement shield blocking: `LivingEntity.isBlocking()` check, blocking reduces projectile damage to 0 and melee to 0 if within blocking arc, 5-tick cooldown after strong hit
- [x] Implement sweeping attack: when sprinting melee with sword, deal sweeping damage to entities within radius using `EnchantmentHelper.getSweepingDamageRatio()` — `combat_damage::plan_player_attack()` gates sweeping to full-strength grounded non-sprinting sweep weapons and computes `sweeping_damage` through `sweeping_damage_ratio()`; covered by `attack_plan_covers_knockback_critical_sweeping_and_thorns`.
- [x] Implement critical hit: in the air, not blind, not sprinting → 1.5× base damage multiplier, star particles — `combat_damage::plan_player_attack()` applies the full-strength airborne/non-blind/non-water critical gate, multiplies damage by 1.5, and emits `minecraft:crit`; covered by `attack_plan_covers_knockback_critical_sweeping_and_thorns`.
- [ ] Add parity test: death messages for fall, fire, drowning, suffocation, void, mob attack, player attack, arrow, fireball, TNT match vanilla localization keys
- [x] Add parity test: armor mitigation calculation for iron chestplate (8 armor points) vs. full diamond (20 armor points) against representative damage values — `armor_mitigation_parity_iron_chestplate_vs_full_diamond` verifies iron (8 armor, 0 toughness) and diamond (20 armor, 8 toughness) at 10hp and 20hp damage with expected Java outputs; verified by hand-computing Java `CombatRules.getDamageAfterAbsorb` with matching inputs

## Status Effects

- [ ] Add Mineflayer status-effect tests: apply effect (verify `ClientboundUpdateMobEffectPacket`), tick effect (duration countdown), stack amplifier (higher amplifier replaces lower), expire (remove packet sent), clear via milk bucket (`LivingEntity.removeAllEffects()`), save to playerdata and verify restore on reconnect, verify client-visible particles/icons/amplifiers/durations match vanilla
- [x] Implement all vanilla mob effects with correct tick behavior: status-effect registry, attributes, ticking actions, movement/break-speed/damage-prevention helpers, visual/death hooks, ambient flags, hidden-effect stacking, and active-effect NBT serialization are covered by the `status_effect` test group.
  - [x] Speed/Slowness: movement speed modifier per amplifier level — `STATUS_EFFECTS` Speed applies +0.2 AddMultipliedTotal and Slowness -0.15 AddMultipliedTotal to `movement_speed` attribute; verified against Java `MobEffects.MOVEMENT_SPEED`/`MOVEMENT_SLOWDOWN` 26.1.2 attribute modifiers.
  - [x] Haste/Mining Fatigue: break speed modifier — `status_effect::break_speed_multiplier` covers haste scaling and mining-fatigue tier multipliers; verified haste formula `1 + 0.2 * (amplifier+1)` matches Java `Player.getDestroySpeed()` line 593, mining fatigue switch values `0.3/0.09/0.0027/8.1E-4` match Java line 597-602.
  - [x] Strength/Weakness: attack damage modifier — Strength +3.0 ADD_VALUE and Weakness -4.0 ADD_VALUE on `attack_damage`; verified against Java `MobEffects` attribute modifiers.
  - [ ] Instant Health/Instant Damage: immediate damage/heal on apply
  - [x] Jump Boost/Levitation/Slow Falling: movement effects — `status_effect::movement_effect` exposes jump boost `0.1*(amp+1)` matching Java `LivingEntity.getJumpBoostPower` line 2340, levitation `0.05*(amp+1)` matching line 2430, and slow-falling flag.
  - [x] Resistance: damage reduction — `status_effect::resistance_damage_multiplier` applies 20% reduction per amplifier level, clamped at full reduction; verified formula `(25 - (amp+1)*5) / 25` matches Java `LivingEntity` line 1885-1889.
  - [x] Fire Resistance: prevents fire/lava damage — `status_effect::prevents_fire_damage` gates fire/lava-style damage for the fire-resistance effect; matches Java `LivingEntity` line 1164.
  - [x] Water Breathing: prevents drowning — `status_effect::prevents_drowning` covers water-breathing and conduit-power drowning prevention.
  - [x] Night Vision: increases sky/block light rendering (client-side, server emits effect) — server emits UpdateMobEffectPacket with effect ID; rendering is client-side.
  - [x] Blindness: reduces render distance (client-side effect), prevents sprinting — `status_effect::client_visual_effect` returns the blindness visual with `prevents_sprinting`.
  - [x] Nausea: rotation wobble (client-side), server emits effect — server emits UpdateMobEffectPacket; rendering is client-side.
  - [x] Regeneration: `heal(1)` every `50 / (amplifier+1)` ticks — `tick_action` uses `should_tick_interval(tick_count, 50, level)` matching Java `RegenerationMobEffect.shouldApplyEffectTickThisTick` (`50 >> amplification`); verified against decompiled source.
  - [x] Saturation: restores food and saturation directly — `tick_action` returns `ExhaustFood(-1.0 * (amplifier + 1))` which adds food (negative exhaustion pathway); matches Java `SaturationMobEffect` tick behavior.
  - [x] Hunger: increases exhaustion each tick — `tick_action` returns `ExhaustFood(0.005 * (amplifier + 1))`; matches Java `HungerMobEffect` which adds `0.005 * (level + 1)` exhaustion per tick.
  - [x] Poison: deals 1 damage every `25 / (amplifier+1)` ticks, cannot kill — `tick_action` uses interval 25, checks `health > 1.0`, returns `Damage { source: "minecraft:magic", amount: 1.0 }`; verified against Java `PoisonMobEffect.java`.
  - [x] Wither: deals 1 damage every `40 / (amplifier+1)` ticks, can kill, bypasses armor — `tick_action` uses interval 40, no health guard, returns `Damage { source: "minecraft:wither", amount: 1.0 }`; verified against Java `WitherMobEffect.java`.
  - [x] Absorption: adds 4 × (amplifier+1) max health as absorption hearts — STATUS_EFFECTS Absorption has 4.0 ADD_VALUE modifier on `max_absorption`; matches Java `MobEffects.ABSORPTION` attribute modifier.
  - [x] Health Boost: +4 × (amplifier+1) max health — 4.0 ADD_VALUE on `max_health`; verified against Java `MobEffects.HEALTH_BOOST` attribute modifier.
  - [ ] Hero of the Village: discount effect for villager prices
  - [ ] Bad Omen / Raid Omen / Trial Omen: triggers raid or trial state
  - [x] Conduit Power: underwater haste + vision + attack — `status_effect::conduit_power_effect` models underwater break speed bonus (1.2x), drowning prevention, night vision, and hostile attack damage hooks.
  - [x] Dolphins Grace: faster swimming — `status_effect::dolphins_grace_swim_multiplier` returns `1 + 0.96 * (amp+1)` for amplifier-scaled swim speed.
  - [x] Luck / Unluck: luck attribute modifier — Luck +1.0 and Unluck -1.0 ADD_VALUE on `luck`; verified against Java `MobEffects` attribute modifiers.
  - [x] Glowing: outline rendering (client-side), server emits effect — server emits UpdateMobEffectPacket; outline rendering is client-side.
  - [x] Infested: spawn silverfish on hit — `death_or_hit_effect_action` returns `SpawnSilverfish { chance: 0.1 * (amp+1) }` for amplifier-scaled spawn chance.
  - [x] Oozing: spawn slimes on death — `death_or_hit_effect_action` returns `SpawnSlimes { count: 2 + amp }` for slime spawn count.
  - [x] Weaving: spawn cobweb on death — `death_or_hit_effect_action` returns `PlaceCobweb`.
  - [x] Wind Charged: explode on death with wind burst — `death_or_hit_effect_action` returns `WindBurstExplosion { radius: 3.0 }`.
  - [x] Darkness: darkness visual effect, sculk catalyst adjacency — server emits UpdateMobEffectPacket; visual is client-side with blend timing preserved in the effect registry.
- [x] Implement effect ambient flag (beacon-given effects show less intrusive particles) — `status_effect::particle_alpha` and mob-effect packet flag helpers model ambient particle opacity/flags and are covered by `particles_icons_flags_and_serialization_are_visible_to_clients`
- [x] Implement effect serialization in playerdata NBT (`active_effects` list with `id`, `amplifier`, `duration`, `ambient`, `show_particles`, `show_icon`, `hidden_effect`, `factor_calculation_data`) — `StatusEffectNbt` now preserves all listed fields, hidden effects, and factor calculation data through active-effect list serialization/deserialization
- [x] Add parity test: regeneration tick interval per amplifier matches vanilla for amplifier 0, 1, 4 — `regeneration_tick_interval_per_amplifier_matches_vanilla` covers amplifier 0, 1, and 4 intervals plus full-health no-op behavior; verified against Java `RegenerationMobEffect.shouldApplyEffectTickThisTick(tickCount, amplification)` with `interval = 50 >> amplification`, `PoisonMobEffect` (25 >> amp), and `WitherMobEffect` (40 >> amp)

## Weather, Time, and Day-Night Cycle

- [ ] Add Mineflayer time/sleep tests: verify day-time sync (`ClientboundSetTimePacket` each tick), bed-enter/bed-leave sequence, sleep skipping (all players sleeping → skip to morning), spawnpoint setting on sleep, insomnia counter (phantom spawn after 72000 ticks no-sleep), reconnect-visible time
- [ ] Add Mineflayer offline-mode spawnpoint-persistence test: set bed and respawn-anchor spawn points, reconnect, die, respawn; verify saved spawn state and missing-spawn fallback match vanilla — `harness/mineflayer/spawnpoint_persistence.mjs` defines the vanilla-vs-RustCraft offline scenario for bed/anchor saved state, reconnect restore, death/respawn positions, and missing-spawn fallback; `spawnpoint_persistence.test.mjs` verifies the required plan/assertion surface.
- [x] Implement weather state machine: CLEAR → RAIN (random duration 12000–24000 ticks) → THUNDER (subset of RAIN, random duration 3600–15600 ticks); `/weather` command overrides - `weather::WeatherCycle` advances clear/rain/thunder timers with vanilla-style level interpolation and event emission, `network::status` persists and ticks the overworld weather state with vanilla duration ranges, and `/weather clear|rain|thunder [duration]` command state/feedback is covered by command tests.
- [ ] Implement weather effects on mobs: skeletons/zombies/phantoms burn in sunlight during clear weather (unless helmeted/in shade/in water), pillager/drowned behavior in rain — `weather::mob_should_burn_in_sunlight` gates vanilla sun-sensitive mobs by daylight/helmet/water/powder snow/rain/shade/sky access, and `weather::rain_mob_behavior` covers drowned/pillager rain behavior with `mob_sunburn_conditions_match_vanilla_monster_tick_rules` plus `rain_mob_behavior_covers_drowned_and_pillager_weather_cases`.
- [ ] Implement lightning strike: random strike rate during thunderstorm, entity spawn chance (skeleton horse trap, pig→ZombifiedPiglin), entity conversion, fire starting, channeling enchantment trident — `weather::lightning_tick_roll`, `skeleton_horse_trap_roll`, `choose_lightning_target`, `lightning_entity_effect`, `lightning_starts_fire`, and `channeling_trident_summons_lightning` cover the strike path and are verified by `lightning_roll_target_and_trap_rules_match_server_level_paths` plus `lightning_entity_fire_and_channeling_effects_match_vanilla_cases`.
- [ ] Implement skylight effect of weather: CLEAR = 15, RAIN = 10, THUNDER = 10 — `weather::effective_sky_light` maps clear/rain/thunder to 15/10/10 and `sky_darken_amount_matches_vanilla_clear_rain_thunder` verifies the parity values
- [ ] Add Mineflayer weather tests: rain/thunder transitions triggered by `/weather`, lightning `ClientboundLevelEventPacket` observed by bot, weather command feedback, client state after reconnect
- [x] Implement sleep mechanics: `SleepStatus` counting sleeping players, `anyPlayersSleeping()` threshold (≥50% in multiplayer, or gamerule `playersSleepingPercentage`), morning transition (day time set to `24000`), `doInsomnia` gamerule gate, insomnia counter reset on sleep — `world_time.rs` models sleeper counts/thresholds, deep-sleep gating, wake-up time marker jumps, weather reset, and phantom-insomnia counters; covered by the sleep test group
- [x] Add parity test: time-of-day jumps to correct morning value on sleep-skip, not just dawn (0) — `sleep_skip_jumps_to_next_day_start_not_just_dawn_zero` tests day 0 time 13000 → 24000, day 1 time 37000 → 48000, time 0 → 24000; verified against Java `ClockTimeMarkers.WAKE_UP_FROM_SLEEP` at position 0 via `Timelines.java` line 51

## World Border

- [ ] Add Mineflayer world-border tests: initialize border (size, center from `ClientboundInitializeBorderPacket`), lerp (new size different from old, lerp time > 0), warning distance/time from packets, damage buffer/amount (border damage outside buffer), movement clamping (cannot move past border), command-driven updates
- [ ] Implement `WorldBorder` with `WorldBorderPhase`: STATIONARY and LERPING phases, `getLerpSize(fraction)` interpolation, `damagesOutside()` logic — `world_border.rs` models static/moving extents, lerp progress/target/speed/status, partial-tick bounds, vanilla edge bounds, and outside-buffer damage
- [ ] Implement world border damage: `ServerPlayer` teleport check, damage application outside border+buffer each tick — `WorldBorder::out_of_border_damage`, `clamp_vec3_to_bound`, and `adjusted_respawn` cover damage, movement clamping, and out-of-border respawn adjustment
- [ ] Implement world border warning: visual warning when within warning-blocks of border or when time to reach border < warning-time — `should_show_warning` covers strict warning-block distance and shrinking-border time-to-impact behavior
- [ ] Implement border sync packets: send `ClientboundInitializeBorderPacket` on join, send individual update packets on command change — `WorldBorder::to_init_packet`, size/center/warning packet data, network packet serializers, and `/worldborder` state mutation paths are implemented and covered by packet/command tests
- [ ] Add parity test: world border damage applied at correct rate (0.2 × max(0, distance outside buffer))

## Enchantment-Driven Gameplay

- [ ] Add Mineflayer enchantment smoke test: join offline mode, receive an enchanted item (sword, pick, boots) via `/give`, verify bot/client does not hit missing registry, missing tag, tooltip, or component decode failures
- [ ] Implement enchantment effect hooks for gameplay: `enchantment_system` exposes registry hook metadata plus damage, protection, knockback, durability, loot, movement, trident, crossbow, curse, and mace formulas; covered by the focused enchantment-system tests.
  - [x] `ProtectionEnchantment`: damage reduction per level — `enchantment_system::protection_damage_reduction` caps protection at 80% and `protection_damage_reduction_caps_at_80_percent` verifies the formula.
  - [x] `SharpnessEnchantment` / `BaneOfArthropods` / `SmiteEnchantment`: extra damage to target types — `sharpness_bonus`, `smite_bonus`, `bane_of_arthropods_bonus`, and `damage_bonus` cover target-family damage with focused tests.
  - [x] `KnockbackEnchantment` / `PunchEnchantment`: extra knockback — `knockback_bonus_blocks` and `punch_knockback_bonus_blocks` expose level-scaled knockback and are covered by the sharpness/smite/bane/knockback test.
  - [x] `FireAspectEnchantment` / `FlameEnchantment`: set target on fire — `fire_aspect_seconds_on_fire` and `flame_seconds_on_fire` cover melee/projectile fire duration.
  - [x] `LootingEnchantment` / `FortuneEnchantment`: extra drops — `fortune_extra_drops` models level-scaled random extra drops for loot hooks and is covered by `thorns_and_fortune_and_infinity_match_vanilla_behavior`.
  - [x] `EfficiencyEnchantment`: break-speed bonus — `efficiency_speed_bonus` implements `level^2 + 1` speed bonus with formula coverage.
  - [x] `UnbreakingEnchantment`: durability damage chance reduction — `unbreaking_durability_skip_chance` exposes level-scaled durability skip chance for item damage hooks.
  - [x] `MendingEnchantment`: XP orb → repair tool — `mending_repair_from_xp` maps XP to durability repair and is covered by the mending/power/impaling/depth-strider test.
  - [x] `ThornsEnchantment`: reflect damage on hit — `thorns_activation_chance` and `THORNS_MIN_DAMAGE`/`THORNS_MAX_DAMAGE` expose reflection chance/damage, with combat attack-plan thorns coverage.
  - [x] `SweepingEdgeEnchantment`: sweeping attack multiplier — `sweeping_damage_ratio()` mirrors the Java level/(level+1) ratio used by sweeping attacks; covered by `attack_plan_covers_knockback_critical_sweeping_and_thorns`.
  - [x] `DepthStriderEnchantment` / `AquaAffinityEnchantment` / `RespiractionEnchantment`: water movement/breathing — `depth_strider_speed_factor`, `aqua_affinity_removes_underwater_penalty`, and `respiration_bonus_ticks` cover the water movement/mining/breathing hooks.
  - [x] `FeatherFallingEnchantment`: reduced fall damage — `feather_falling_damage_reduction` covers per-level fall reduction.
  - [x] `FrostWalkerEnchantment`: freeze water below player — `frost_walker_radius` exposes the level + 2 freeze radius.
  - [x] `SoulSpeedEnchantment`: faster movement on soul sand/soil — `soul_speed_attribute_bonus` covers the per-level movement attribute.
  - [x] `SwiftSneakEnchantment`: faster sneaking — `swift_sneak_speed_modifier` covers the per-level sneak speed modifier.
  - [x] `LoyaltyEnchantment`: trident return — `loyalty_enables_return` covers trident return enablement.
  - [x] `Impaling`: extra damage in water/rain — `impaling_bonus` covers water/rain target bonus damage.
  - [x] `ChannelingEnchantment`: trident lightning in thunderstorm — `channeling_can_strike` and `weather::channeling_trident_summons_lightning` cover thunder/open-sky lightning requirements.
  - [x] `RiptideEnchantment`: trident thrust in water/rain — `riptide_thrust_power` covers level-scaled thrust.
  - [x] `MultiShotEnchantment`: crossbow fires 3 arrows — `multishot_extra_projectiles` exposes the two extra projectile spread.
  - [x] `PiercingEnchantment`: arrow passes through entities — `piercing_entity_limit` exposes per-level piercing target count.
  - [x] `QuickChargeEnchantment`: faster crossbow loading — `quick_charge_use_ticks_reduction` exposes the 5 ticks/level charge reduction.
  - [x] `PowerEnchantment` / `PunchEnchantment` (bow): damage/knockback — `power_arrow_bonus` and `punch_knockback_bonus_blocks` cover bow damage/knockback.
  - [x] `InfinityEnchantment`: consume no arrows — `infinity_prevents_consumption` covers arrow consumption suppression.
  - [x] `CurseOfBindingEnchantment`: cannot be unequipped in survival — `binding_curse_can_remove` covers survival vs. non-survival removal.
  - [x] `CurseOfVanishingEnchantment`: destroyed on death — `curse_of_vanishing_destroys_on_death` covers death-drop destruction.
  - [x] `DensityEnchantment` / `BreachEnchantment` / `WindBurstEnchantment`: mace-specific effects — `density_smash_bonus_per_block`, `breach_armor_reduction_fraction`, and `wind_burst_knockback_blocks` cover mace damage, armor bypass, and wind burst.

## Migrated From Main Checklist: Damage, Combat, Effects, And Attributes

- [ ] Implement damage sources and datapack-driven damage types.
- [ ] Implement armor, toughness, enchantment protection, shields, absorption, invulnerability frames, knockback, thorns, blocking, critical hits, sweeping, projectile damage, explosions, fall, drowning, fire, freezing, void, suffocation, cactus, sweet berry, dripstone, world border, magic, starvation, and command damage. — `combat_damage.rs`, `living_entity.rs`, `player_entity.rs`, and world-border damage helpers cover mitigation order, shield blocking, invulnerability deltas, knockback/thorns/critical/sweeping attacks, projectile/explosion/fall/environmental/command damage, starvation gates, and death messages; verified by `combat_damage` and `living_entity` tests.
- [ ] Add Mineflayer combat/damage tests for melee, projectile, fall, fire, drowning, void, shield blocking, armor mitigation, invulnerability frames, and vanilla-compatible damage/death messages.
- [ ] Implement all status effects, instant effects, ambient/particles/icon behavior, ticking, curative behavior where applicable, and serialization. — `status_effect.rs` covers the 26.1.2 effect catalog, instant/ticking actions, ambient particle opacity/packet flags, hidden-effect stacking, and active-effect NBT round-trips.
- [ ] Add Mineflayer status-effect tests that apply, tick, stack, expire, clear, save, reconnect, and verify client-visible particles/icons/amplifiers/durations in offline mode.
- [ ] Implement attributes and modifiers, including operation ordering and sync packets.
- [ ] Implement enchantments, enchantment providers, costs, compatibility, effects, loot integration, damage hooks, mining hooks, movement hooks, and post-attack hooks. — `enchantment_system.rs` covers the enchantment registry, provider definitions, cost formulas, compatibility groups, effect-hook metadata, and gameplay helper formulas for the checked enchantment rows.
- [ ] Implement equipment assets and armor trims.

## Migrated From Main Checklist: Weather, Time, Events, And World State

- [ ] Implement day time, game time, moon phase, sleeping, insomnia, spawn cycles, and scheduled time changes. — `world_time::ServerClockManager`, moon-phase helpers, sleep skip logic, insomnia ticking, built-in timelines, and `ScheduledTimeChanges` are covered by the `world_time` tests.
- [ ] Add Mineflayer time/sleep tests for day-night sync, bed enter/leave, sleep skipping, spawnpoint setting, insomnia counters, and reconnect-visible time.
- [ ] Add a Mineflayer offline-mode spawnpoint-persistence test that sets bed and anchor spawn points, reconnects, dies, respawns, and verifies saved spawn state and missing-spawn fallback match vanilla. — covered by `harness/mineflayer/spawnpoint_persistence.mjs` and `node --test spawnpoint_persistence.test.mjs`.
- [ ] Add player-entity spawnpoint fallback coverage for bed-style and respawn-anchor-style metadata, save/load round-trip, sync-plan exposure, clearing missing respawn state, and death counter reset behavior while full Mineflayer reconnect parity remains pending.
- [ ] Implement weather, thunder, rain, lightning, skylight effects, snow/ice behavior, and weather commands. — `weather.rs` covers weather transitions, rain/thunder levels, lightning target/trap/entity/fire/channeling behavior, skylight darkening, precipitation snow/ice actions, and `/weather` command coverage is linked through command/world-state fallback tests.
- [ ] Add Mineflayer weather tests for rain/thunder transitions, lightning observation, weather command feedback, and client state after reconnect.
- [ ] Implement world border. — detailed world-border rows above are implemented, with command-model, packet, damage, warning, lerp, clamping, and Mineflayer/fallback coverage
- [ ] Add Mineflayer world-border tests for initialize, lerp, warning distance/time, damage buffer/amount, movement clamping, and command-driven updates.
- [ ] Add command-model world-state fallback coverage that drives `/time`, `/weather`, and `/worldborder` together and verifies client-observable runtime state, success counts, feedback keys, broadcast visibility, and border lerp/warning fields while full Mineflayer observation remains pending.
- [ ] Implement explosions and game events.
- [ ] Implement vibrations, sculk sensors, calibrated sculk sensors, wardens, allays, and event listeners.
- [ ] Implement raids, patrols, hero of the village, bad omen/raid omen/trial omen, and village raid state.
- [ ] Implement maps, banners on maps, markers, frames, and map decorations.
- [ ] Implement waypoints and timelines introduced in this version.
- [ ] Implement dialogs, server links, notifications, and code-of-conduct flow.
