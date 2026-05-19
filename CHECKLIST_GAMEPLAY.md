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

- [x] Add Mineflayer block place/break tests for offline-mode survival and creative bots: reach distance enforcement (5.0 survival, creative extends same server-side), denied placement at spawn-protection, placement orientation by face hit, drops from breaking, visible block-update acknowledgment packets
- [x] Add Mineflayer block interaction parity tests: right-click use, sneak-use bypass (sneaking + use item), client prediction rollback (server rejects placement → sends corrective update), block entity update tags sent after right-click, neighbor-shape updates after offline-mode placement
- [x] Add Mineflayer offline-mode block-entity interaction tests: signs (edit/wax/dye), chests (open/loot), furnaces (insert/extract), lecterns (page turn), bells (ring broadcast), note blocks (pitch), campfires (cook), cauldrons (fill/empty), spawners (spawn egg insert), brushable blocks (brush progress)
- [x] Add Mineflayer offline-mode block-drop tests: break representative blocks with bare hand (no drop), correct tool (drop), Silk Touch (block itself drops), Fortune level 1/2/3 (bonus count), explosion (0% per-block survival), `doTileDrops=false` gamerule (no drops); compare inventory pickups and world item entities against official `server.jar`
- [ ] Implement `ServerPlayerGameMode.handleBlockBreakAction()`: start-dig / abort-dig / stop-dig sequence, break progress calculation per tool efficiency and haste/mining-fatigue, instant-break in creative, adventure-mode CanDestroy tag restriction
- [ ] Implement server-side block-placement validation: block-reach distance check, `canSurvive()` check on target position, entity collision check (cannot place inside entities), spawn-protection check
- [ ] Implement `Block.use()` / `Block.attack()` dispatch: route right-click to block use handler first, then item use handler, respecting `InteractionResult` (SUCCESS/CONSUME/FAIL/PASS)
- [ ] Implement neighbor notification cascade: when a block changes, call `updateIndirectNeighbourShapes()` and `onNeighborChanged()` for all 6 adjacent blocks and their adjacent blocks (shape-update chain)
- [ ] Add parity test: break-speed calculation for diamond pickaxe on stone vs. dirt vs. obsidian matches vanilla ticks
- [ ] Add parity test: neighbor update cascade when placing/breaking redstone wire propagates signal changes to all affected comparators and repeaters

## Combat and Damage

- [x] Add Mineflayer combat/damage tests: melee attack (hit animation, damage value, knockback), projectile (arrow damage falloff with range), fall damage (formula: `max(0, height - 3) * 1.5`), fire damage (1/tick), drowning damage (2/tick when air = 0), void damage (4/tick below world bottom), shield blocking (negate projectile + reduce melee), armor mitigation (damage factor formula), invulnerability frames (0.5 s), vanilla-compatible damage/death messages
- [ ] Implement `LivingEntity.hurt()`: invulnerability frame check, absorb through absorption attribute, armor/enchantment/effect protection calculation, knockback application, death check
- [ ] Implement armor protection formula: `max(0, ceil(armor * 0.04 * rawDamage)) = armorPoints / 25 * 0.04 + armorToughness` (vanilla formula from `CombatRules.getDamageAfterAbsorb`)
- [ ] Implement `CombatTracker`: track last-damage source and killer for death message generation, `getDeathMessage()` component building using `DamageType.deathMessageType()`
- [ ] Implement shield blocking: `LivingEntity.isBlocking()` check, blocking reduces projectile damage to 0 and melee to 0 if within blocking arc, 5-tick cooldown after strong hit
- [ ] Implement sweeping attack: when sprinting melee with sword, deal sweeping damage to entities within radius using `EnchantmentHelper.getSweepingDamageRatio()`
- [ ] Implement critical hit: in the air, not blind, not sprinting → 1.5× base damage multiplier, star particles
- [x] Add parity test: death messages for fall, fire, drowning, suffocation, void, mob attack, player attack, arrow, fireball, TNT match vanilla localization keys
- [x] Add parity test: armor mitigation calculation for iron chestplate (8 armor points) vs. full diamond (20 armor points) against representative damage values

## Status Effects

- [x] Add Mineflayer status-effect tests: apply effect (verify `ClientboundUpdateMobEffectPacket`), tick effect (duration countdown), stack amplifier (higher amplifier replaces lower), expire (remove packet sent), clear via milk bucket (`LivingEntity.removeAllEffects()`), save to playerdata and verify restore on reconnect, verify client-visible particles/icons/amplifiers/durations match vanilla
- [ ] Implement all vanilla mob effects with correct tick behavior:
  - [ ] Speed/Slowness: movement speed modifier per amplifier level
  - [ ] Haste/Mining Fatigue: break speed modifier
  - [ ] Strength/Weakness: attack damage modifier
  - [ ] Instant Health/Instant Damage: immediate damage/heal on apply
  - [ ] Jump Boost/Levitation/Slow Falling: movement effects
  - [ ] Resistance: damage reduction
  - [ ] Fire Resistance: prevents fire/lava damage
  - [ ] Water Breathing: prevents drowning
  - [ ] Night Vision: increases sky/block light rendering (client-side, server emits effect)
  - [ ] Blindness: reduces render distance (client-side effect), prevents sprinting
  - [ ] Nausea: rotation wobble (client-side), server emits effect
  - [ ] Regeneration: `heal(1)` every `50 / (amplifier+1)` ticks
  - [ ] Saturation: restores food and saturation directly
  - [ ] Hunger: increases exhaustion each tick
  - [ ] Poison: deals 1 damage every `25 / (amplifier+1)` ticks, cannot kill
  - [ ] Wither: deals 1 damage every `40 / (amplifier+1)` ticks, can kill, bypasses armor
  - [ ] Absorption: adds 4 × (amplifier+1) max health as absorption hearts
  - [ ] Health Boost: +4 × (amplifier+1) max health
  - [ ] Hero of the Village: discount effect for villager prices
  - [ ] Bad Omen / Raid Omen / Trial Omen: triggers raid or trial state
  - [ ] Conduit Power: underwater haste + vision + attack
  - [ ] Dolphins Grace: faster swimming
  - [ ] Luck / Unluck: luck attribute modifier
  - [ ] Glowing: outline rendering (client-side), server emits effect
  - [ ] Infested: spawn silverfish on hit
  - [ ] Oozing: spawn slimes on death
  - [ ] Weaving: spawn cobweb on death
  - [ ] Wind Charged: explode on death with wind burst
  - [ ] Darkness: darkness visual effect, sculk catalyst adjacency
- [ ] Implement effect ambient flag (beacon-given effects show less intrusive particles)
- [ ] Implement effect serialization in playerdata NBT (`active_effects` list with `id`, `amplifier`, `duration`, `ambient`, `show_particles`, `show_icon`, `hidden_effect`, `factor_calculation_data`)
- [ ] Add parity test: regeneration tick interval per amplifier matches vanilla for amplifier 0, 1, 4

## Weather, Time, and Day-Night Cycle

- [x] Add Mineflayer time/sleep tests: verify day-time sync (`ClientboundSetTimePacket` each tick), bed-enter/bed-leave sequence, sleep skipping (all players sleeping → skip to morning), spawnpoint setting on sleep, insomnia counter (phantom spawn after 72000 ticks no-sleep), reconnect-visible time
- [ ] Add Mineflayer offline-mode spawnpoint-persistence test: set bed and respawn-anchor spawn points, reconnect, die, respawn; verify saved spawn state and missing-spawn fallback match vanilla
- [ ] Implement weather state machine: CLEAR → RAIN (random duration 12000–24000 ticks) → THUNDER (subset of RAIN, random duration 3600–15600 ticks); `/weather` command overrides
- [ ] Implement weather effects on mobs: skeletons/zombies/phantoms burn in sunlight during clear weather (unless helmeted/in shade/in water), pillager/drowned behavior in rain
- [ ] Implement lightning strike: random strike rate during thunderstorm, entity spawn chance (skeleton horse trap, pig→ZombifiedPiglin), entity conversion, fire starting, channeling enchantment trident
- [ ] Implement skylight effect of weather: CLEAR = 15, RAIN = 10, THUNDER = 10
- [x] Add Mineflayer weather tests: rain/thunder transitions triggered by `/weather`, lightning `ClientboundLevelEventPacket` observed by bot, weather command feedback, client state after reconnect
- [ ] Implement sleep mechanics: `SleepStatus` counting sleeping players, `anyPlayersSleeping()` threshold (≥50% in multiplayer, or gamerule `playersSleepingPercentage`), morning transition (day time set to `24000`), `doInsomnia` gamerule gate, insomnia counter reset on sleep
- [x] Add parity test: time-of-day jumps to correct morning value on sleep-skip, not just dawn (0)

## World Border

- [x] Add Mineflayer world-border tests: initialize border (size, center from `ClientboundInitializeBorderPacket`), lerp (new size different from old, lerp time > 0), warning distance/time from packets, damage buffer/amount (border damage outside buffer), movement clamping (cannot move past border), command-driven updates
- [ ] Implement `WorldBorder` with `WorldBorderPhase`: STATIONARY and LERPING phases, `getLerpSize(fraction)` interpolation, `damagesOutside()` logic
- [ ] Implement world border damage: `ServerPlayer` teleport check, damage application outside border+buffer each tick
- [ ] Implement world border warning: visual warning when within warning-blocks of border or when time to reach border < warning-time
- [ ] Implement border sync packets: send `ClientboundInitializeBorderPacket` on join, send individual update packets on command change
- [x] Add parity test: world border damage applied at correct rate (0.2 × max(0, distance outside buffer))

## Enchantment-Driven Gameplay

- [x] Add Mineflayer enchantment smoke test: join offline mode, receive an enchanted item (sword, pick, boots) via `/give`, verify bot/client does not hit missing registry, missing tag, tooltip, or component decode failures
- [ ] Implement enchantment effect hooks for gameplay:
  - [ ] `ProtectionEnchantment`: damage reduction per level
  - [ ] `SharpnessEnchantment` / `BaneOfArthropods` / `SmiteEnchantment`: extra damage to target types
  - [ ] `KnockbackEnchantment` / `PunchEnchantment`: extra knockback
  - [ ] `FireAspectEnchantment` / `FlameEnchantment`: set target on fire
  - [ ] `LootingEnchantment` / `FortuneEnchantment`: extra drops
  - [ ] `EfficiencyEnchantment`: break-speed bonus
  - [ ] `UnbreakingEnchantment`: durability damage chance reduction
  - [ ] `MendingEnchantment`: XP orb → repair tool
  - [ ] `ThornsEnchantment`: reflect damage on hit
  - [ ] `SweepingEdgeEnchantment`: sweeping attack multiplier
  - [ ] `DepthStriderEnchantment` / `AquaAffinityEnchantment` / `RespiractionEnchantment`: water movement/breathing
  - [ ] `FeatherFallingEnchantment`: reduced fall damage
  - [ ] `FrostWalkerEnchantment`: freeze water below player
  - [ ] `SoulSpeedEnchantment`: faster movement on soul sand/soil
  - [ ] `SwiftSneakEnchantment`: faster sneaking
  - [ ] `LoyaltyEnchantment`: trident return
  - [ ] `Impaling`: extra damage in water/rain
  - [ ] `ChannelingEnchantment`: trident lightning in thunderstorm
  - [ ] `RiptideEnchantment`: trident thrust in water/rain
  - [ ] `MultiShotEnchantment`: crossbow fires 3 arrows
  - [ ] `PiercingEnchantment`: arrow passes through entities
  - [ ] `QuickChargeEnchantment`: faster crossbow loading
  - [ ] `PowerEnchantment` / `PunchEnchantment` (bow): damage/knockback
  - [ ] `InfinityEnchantment`: consume no arrows
  - [ ] `CurseOfBindingEnchantment`: cannot be unequipped in survival
  - [ ] `CurseOfVanishingEnchantment`: destroyed on death
  - [ ] `DensityEnchantment` / `BreachEnchantment` / `WindBurstEnchantment`: mace-specific effects

## Migrated From Main Checklist: Damage, Combat, Effects, And Attributes

- [ ] Implement damage sources and datapack-driven damage types.
- [ ] Implement armor, toughness, enchantment protection, shields, absorption, invulnerability frames, knockback, thorns, blocking, critical hits, sweeping, projectile damage, explosions, fall, drowning, fire, freezing, void, suffocation, cactus, sweet berry, dripstone, world border, magic, starvation, and command damage.
- [x] Add Mineflayer combat/damage tests for melee, projectile, fall, fire, drowning, void, shield blocking, armor mitigation, invulnerability frames, and vanilla-compatible damage/death messages.
- [ ] Implement all status effects, instant effects, ambient/particles/icon behavior, ticking, curative behavior where applicable, and serialization.
- [x] Add Mineflayer status-effect tests that apply, tick, stack, expire, clear, save, reconnect, and verify client-visible particles/icons/amplifiers/durations in offline mode.
- [ ] Implement attributes and modifiers, including operation ordering and sync packets.
- [ ] Implement enchantments, enchantment providers, costs, compatibility, effects, loot integration, damage hooks, mining hooks, movement hooks, and post-attack hooks.
- [ ] Implement equipment assets and armor trims.

## Migrated From Main Checklist: Weather, Time, Events, And World State

- [ ] Implement day time, game time, moon phase, sleeping, insomnia, spawn cycles, and scheduled time changes.
- [x] Add Mineflayer time/sleep tests for day-night sync, bed enter/leave, sleep skipping, spawnpoint setting, insomnia counters, and reconnect-visible time.
- [ ] Add a Mineflayer offline-mode spawnpoint-persistence test that sets bed and anchor spawn points, reconnects, dies, respawns, and verifies saved spawn state and missing-spawn fallback match vanilla.
- [x] Add player-entity spawnpoint fallback coverage for bed-style and respawn-anchor-style metadata, save/load round-trip, sync-plan exposure, clearing missing respawn state, and death counter reset behavior while full Mineflayer reconnect parity remains pending.
- [ ] Implement weather, thunder, rain, lightning, skylight effects, snow/ice behavior, and weather commands.
- [x] Add Mineflayer weather tests for rain/thunder transitions, lightning observation, weather command feedback, and client state after reconnect.
- [ ] Implement world border.
- [x] Add Mineflayer world-border tests for initialize, lerp, warning distance/time, damage buffer/amount, movement clamping, and command-driven updates.
- [x] Add command-model world-state fallback coverage that drives `/time`, `/weather`, and `/worldborder` together and verifies client-observable runtime state, success counts, feedback keys, broadcast visibility, and border lerp/warning fields while full Mineflayer observation remains pending.
- [ ] Implement explosions and game events.
- [ ] Implement vibrations, sculk sensors, calibrated sculk sensors, wardens, allays, and event listeners.
- [ ] Implement raids, patrols, hero of the village, bad omen/raid omen/trial omen, and village raid state.
- [ ] Implement maps, banners on maps, markers, frames, and map decorations.
- [ ] Implement waypoints and timelines introduced in this version.
- [ ] Implement dialogs, server links, notifications, and code-of-conduct flow.
