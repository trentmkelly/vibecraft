# Mob and Entity Behavior Checklist

## Relevant Java Source Files

- `decompiled-server-26.1.2/net/minecraft/world/entity/` — base entity hierarchy (`Entity`, `LivingEntity`, `Mob`, `PathfinderMob`, `TamableAnimal`, etc.)
- `decompiled-server-26.1.2/net/minecraft/world/entity/monster/` — all hostile mobs (top-level)
- `decompiled-server-26.1.2/net/minecraft/world/entity/monster/breeze/` — Breeze
- `decompiled-server-26.1.2/net/minecraft/world/entity/monster/creaking/` — Creaking
- `decompiled-server-26.1.2/net/minecraft/world/entity/monster/hoglin/` — Hoglin
- `decompiled-server-26.1.2/net/minecraft/world/entity/monster/illager/` — Illager family (Evoker, Pillager, Vindicator, Illusioner)
- `decompiled-server-26.1.2/net/minecraft/world/entity/monster/piglin/` — Piglin/PiglinBrute
- `decompiled-server-26.1.2/net/minecraft/world/entity/monster/skeleton/` — Skeleton, Stray, WitherSkeleton, Bogged, Parched
- `decompiled-server-26.1.2/net/minecraft/world/entity/monster/spider/` — Spider, CaveSpider
- `decompiled-server-26.1.2/net/minecraft/world/entity/monster/warden/` — Warden, WardenAi, AngerManagement
- `decompiled-server-26.1.2/net/minecraft/world/entity/monster/zombie/` — Zombie, Drowned, Husk, ZombieVillager, ZombifiedPiglin
- `decompiled-server-26.1.2/net/minecraft/world/entity/boss/` — EnderDragon, WitherBoss, EndCrystal, dragon phase instances
- `decompiled-server-26.1.2/net/minecraft/world/entity/npc/` — Villager, WanderingTrader, VillagerProfession, WanderingTraderSpawner
- `decompiled-server-26.1.2/net/minecraft/world/entity/animal/` — all passive/neutral animals
- `decompiled-server-26.1.2/net/minecraft/world/entity/raid/` — Raid, Raider, PatrolSpawner
- `decompiled-server-26.1.2/net/minecraft/world/entity/ai/` — goal system, brain, sensors, memories, activities, navigation, gossip
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/PhantomSpawner.java` — phantom spawn logic
- `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/PatrolSpawner.java` — patrol spawn logic
- `decompiled-server-26.1.2/src/spawning.rs` — RustCraft spawning
- `decompiled-server-26.1.2/src/mob_family.rs` — RustCraft mob family stubs
- `decompiled-server-26.1.2/src/ai_system.rs` — RustCraft AI system

## Boss Families

### Ender Dragon
- [ ] Implement `EnderDragonPhaseManager` and all 11 phase instances: `DragonChargePlayerPhase`, `DragonDeathPhase`, `DragonHoldingPatternPhase`, `DragonHoverPhase`, `DragonLandingApproachPhase`, `DragonLandingPhase`, `DragonSittingAttackingPhase`, `DragonSittingFlamingPhase`, `DragonSittingScanningPhase`, `DragonStrafePlayerPhase`, `DragonTakeoffPhase`
- [ ] Implement `DragonFlightHistory` path recording and head/neck interpolation
- [ ] Implement `EnderDragonPart` multi-hitbox (head, neck, body, left/right wings, left/right wing tips, tail 1/2/3)
- [ ] Implement dragon fireball acid attack and lingering cloud placement
- [ ] Implement dragon crystal healing: `EndCrystal` health regeneration, crystal beam rendering metadata
- [ ] Implement dragon death sequence: explosion cluster, XP orb drops, exit portal creation (bedrock + end stone frame + end portal blocks), dragon egg placement on first kill
- [ ] Implement dragon respawn sequence: four end crystals placed, dragon summon beam, fight state reset
- [ ] Implement `EndDragonFight` per-dimension state: first-kill/respawn tracking, gateway counter, portal position persistence in `level.dat`
- [ ] Implement dragon bossbar: health %, name component, `BossEvent.BossBarColor.PINK`, overlay `PROGRESS`
- [ ] Add parity test: dragon phase transitions match vanilla sequence for cold start (no crystals) vs. crystal-assisted fight
- [ ] Add parity test: exit portal block layout and egg position after first dragon kill
- [ ] Add parity test: dragon respawn beacon positions and timing relative to crystal placement

### Wither Boss
- [ ] Implement `WitherBoss` three-head targeting (distinct target per head), skull projectile alternation
- [ ] Implement `WitherSkull` projectile: normal (gray, faster, larger blast) and blue/charged (breaks hard blocks)
- [ ] Implement wither charge attack at target once below half health
- [ ] Implement wither shield/invulnerability below half health (only damaged by explosions and skulls)
- [ ] Implement wither self-healing over time and on skull hit
- [ ] Implement wither "wither effect" aura on hit
- [ ] Implement wither death sequence: explosion, nether star drop, `BossEvent` removal
- [ ] Implement wither summoning precondition check (T-shape soul sand/soil + 3 wither skulls, not in superflat End)
- [ ] Implement wither bossbar: `BossEvent.BossBarColor.PURPLE`, health %
- [ ] Implement wither block-breaking behavior (breaks most blocks in path)
- [ ] Add parity test: wither skull target assignment, blue skull vs. normal skull NBT
- [ ] Add parity test: shield phase invulnerability threshold and self-heal rate

## Monster Families

### Top-Level Monsters
- [ ] Implement `Blaze`: fire charge burst (3 fireballs per attack cycle), hover flight AI, fire immunity, drop (blaze rod)
- [ ] Implement `Creeper`: fuse countdown (`fuseTime`), explosion on fuse complete, `ignited` NBT (flint+steel trigger), charged variant (lightning conversion), `ExplosionPower` scaling for charged
- [ ] Implement `ElderGuardian`: elder curse (Mining Fatigue III, 60s) within 50 blocks, beam laser at aquatic player targets, spike thorns passive
- [ ] Implement `EnderMan`: holding block (pickup/place logic), teleport on water/rain, look-at-eyes anger trigger, teleport-away from projectiles, scream sound on anger
- [ ] Implement `Endermite`: spawn from ender pearl (5% chance), 2-minute despawn timer, attacked by endermen
- [ ] Implement `Ghast`: attack cycle (charge fireball, shoot), large hitbox (4×4×4), fire immunity, scream/shoot sounds, deflectable fireball by projectiles/swords
- [ ] Implement `Giant`: legacy oversized zombie (no natural spawn, only `/summon`), 12-block height, no AI goals
- [ ] Implement `Guardian`: laser beam targeting (beam charge, inflict damage at full charge), spikes thorns, elder spawning on death
- [ ] Implement `MagmaCube`: split into smaller cubes on death (size-1 → 2–4 smaller), slime-family movement, fire immunity
- [ ] Implement `Phantom`: insomnia prerequisite spawn (no sleep >3 days), circling-swoop attack, burn in daylight, phantom membrane drop, `MemoryModuleType.NEAREST_PLAYERS` usage
- [ ] Implement `Ravager`: stomp attack (AoE damage + slow), roar stun after rider dismounts, grieving leaves/crops in path
- [ ] Implement `Shulker`: attached face (6 directions), shell open/close, homing `ShulkerBullet` projectile, teleport on damage, levitation effect on hit, color variants, invisible variant
- [ ] Implement `Silverfish`: infest stone/cobblestone/brick blocks on death, emergency-call behavior waking nearby infested blocks
- [ ] Implement `Slime`: size variants (1/2/4), split on death, water avoidance, harmless when `size <= 1`
- [ ] Implement `Strider`: lava walking, saddle+ride with carrot-on-a-stick steering, trembling cold-biome state, attraction to warped fungus
- [ ] Implement `Vex`: summoned by evoker (max 3 per evoker), phase through solid blocks, 30–119 tick lifespan after summon
- [ ] Implement `Witch`: splash potion attack (harm/slowness/weakness/poison selection by context), self-healing with instant-health/fire-resistance/water-breathing/swiftness potions, raid rider on Ravager
- [ ] Implement `Zoglin`: permanent hostile (converted from Hoglin entering Overworld/End), knockback attack, immunity to regular zombification

### Breeze
- [ ] Implement `Breeze` locomotion: `LongJump` behavior with momentum tracking, `Slide` ground-movement behavior
- [ ] Implement Breeze wind charge attack: `Shoot` (aim-and-fire when clear line of sight), `ShootWhenStuck` (fire when stuck)
- [ ] Implement `BreezeUtil`: deflect nearby projectiles (arrows, etc.) on breeze wind charge detonation
- [ ] Implement `BreezeAi` brain behavior tree and memory modules
- [ ] Add parity test: breeze long-jump trajectory, wind charge knockback radius, projectile deflection

### Creaking
- [ ] Implement `Creaking`: frozen-when-observed mechanic (cannot move while any player has line-of-sight), invulnerable while tethered to active `CreakingHeart` block
- [ ] Implement `Creaking` tether: linked to nearest `CreakingHeartBlockEntity`, dies when heart is destroyed or deactivated at night end
- [ ] Implement `CreakingAi` brain: activation/deactivation schedule tied to creaking heart state, chase-player goal when not observed
- [ ] Implement Creaking scratch attack on contact
- [ ] Add parity test: creaking freeze on player gaze, movement resume on gaze loss, death on heart break

### Hoglin
- [ ] Implement `Hoglin`: pack hunting (attack nearest non-hoglin mob or player), flee from warped fungi, respawn anchors, nether portals, and piglins
- [ ] Implement `HoglinBase` conversion: Hoglin → Zoglin when entering Overworld or End
- [ ] Implement `HoglinAi` brain: pack behavior, fleeing memories
- [ ] Add parity test: hoglin attack range, flee distance from warped fungi, zoglin conversion timing

### Illager Family
- [ ] Implement `AbstractIllager`: shared attributes, ominous-banner captain drop (first in raid wave), `Illager.IllagerArmPose` metadata sync
- [ ] Implement `Evoker`: fang-attack pattern (line of summoned `EvokerFangs`), vex summoning (3 at 60% health threshold), wololo conversion of blue sheep → red sheep
- [ ] Implement `Illusioner`: invisibility spell on player targeting, mirage clone spawning, blindness arrow
- [ ] Implement `Pillager`: crossbow attack, captain-banner pickup on kill, patrol leader behavior
- [ ] Implement `SpellcasterIllager`: `IllagerSpell` enum, casting animation, cooldown between spell types
- [ ] Implement `Vindicator`: axe melee, Johnny behavior (`CustomName == "Johnny"` attacks all mobs)
- [ ] Add parity test: evoker fang line placement relative to target, vex count per evoker
- [ ] Add parity test: raid wave composition per difficulty for each wave (illager types, counts)

### Piglin Family
- [ ] Implement `AbstractPiglin`: zombification when 15 s in Overworld/End
- [ ] Implement `Piglin`: bartering mechanic (gold ingot throw → loot drop from `data/minecraft/loot_table/gameplay/piglin_bartering.json`), admire-item behavior, anger-on-gold-theft, crossbow attack, dance-to-jukebox memory, baby piglin
- [ ] Implement `PiglinAi` full behavior tree: `StartAdmiringItemIfSeen`, `StopAdmiringIfItemTooFarAway`, `StopAdmiringIfTiredOfTryingToReachItem`, `StopHoldingItemIfNoLongerAdmiring`, `RememberIfHoglinWasKilled`, `StartHuntingHoglin`
- [ ] Implement `PiglinBrute`: no bartering, enhanced attack, always hostile to players (never passive)
- [ ] Implement `PiglinBruteAi` brain behavior tree
- [ ] Add parity test: piglin bartering loot table output distribution across gold throws
- [ ] Add parity test: piglin anger triggers (opening gold chest, picking up gold in front of piglin)

### Skeleton Family
- [ ] Implement `AbstractSkeleton`: bow/crossbow ranging with `RangedAttackGoal`, strafing behavior, melee fallback when close
- [ ] Implement `Skeleton`: burn in daylight (unless wearing helmet), conversion to Stray in powder snow (over 140 ticks)
- [ ] Implement `Stray`: slow-IV tipped arrows, spawns in cold/snowy biomes
- [ ] Implement `WitherSkeleton`: fire immunity, wither-effect sword melee, fortress-only natural spawn
- [ ] Implement `Bogged`: poison-tipped arrows, spawns in swamp/mangrove swamp
- [ ] Implement `Parched`: fire immunity variant (desert environment)
- [ ] Add parity test: skeleton arrow accuracy at various distances, stray powder-snow conversion tick count

### Spider Family
- [ ] Implement `Spider`: wall-climbing behavior (applies NoGravity+climb velocity on wall contact), spider-jockey (1% chance with skeleton mount), neutral in daylight, hostile in dark
- [ ] Implement `CaveSpider`: smaller bounding box, poison bite (7 ticks easy, 15 normal, not hard)
- [ ] Add parity test: spider wall-climb velocity, cave spider poison duration by difficulty

### Warden
- [ ] Implement `Warden` anger management: `AngerLevel` enum (CALM/AGITATED/ANGRY), `AngerManagement` per-entity anger tracker with decay
- [ ] Implement `WardenAi` brain behaviors: vibration listening, sniff (AoE suspicion when suspect nearby), investigate (move to vibration source), roar (before first attack), sonic boom (ranged, bypasses armor), dig-in/dig-out animations
- [ ] Implement warden sonic boom: `DamageType` that ignores armor, knockback, 2.5-block radius
- [ ] Implement `WardenSpawnTracker`: warning-level escalation on shriek, cooldown, persist in player NBT, `/warden_spawn_tracker` command hooks
- [ ] Add parity test: warden anger escalation from vibration/player detection, dig-out animation frames
- [ ] Add parity test: sonic boom ignores armor calculation, warden spawn threshold (3 shrieks within cooldown window)

### Zombie Family
- [ ] Implement `Zombie`: drowning conversion to Drowned (baby cannot convert), baby zombie (5% chance, 25×faster), reinforcement summoning on hit (when `doMobSpawning` gamerule is on)
- [ ] Implement `Drowned`: trident attack AI (throw trident only when in water/rain), swim AI, nautilus-shell attraction
- [ ] Implement `Husk`: desert biome only, does not burn in daylight, hunger effect on hit
- [ ] Implement `ZombieVillager`: `conversionTime` countdown when given Weakness + golden apple, curing discounts on trade prices, profession/village preserved through conversion
- [ ] Implement `ZombifiedPiglin`: neutral until hit, group-anger propagation (nearby ZombifiedPiglins within 67 blocks angered), portal-spawn on nether entry
- [ ] Add parity test: zombie villager cure duration and resulting price discount values
- [ ] Add parity test: zombified piglin anger group radius and anger decay time

## Animal Families

- [ ] Implement `Allay`: follow note-block memory, collect items matching held item, duplicate on jukebox play + amethyst shard, dancing-to-jukebox memory
- [ ] Implement `Armadillo`: roll into ball on threat (player sprint/mount approach), scute dropping, wolf-armor crafting ingredient source
- [ ] Implement `Axolotl`: play-dead behavior (`DeathAnimation`), attack aquatic hostiles (guardians, drowned, etc.), `BucketableEntity` bucket capture, axolotl color variants (5 types including rare blue)
  - [x] Implement Java-matched `Axolotl` color variant model: lucy/wild/gold/cyan common variants, rare blue variant, default lucy, and 1/1200 rare breeding chance
  - [x] Implement Java-matched `Axolotl` play-dead gates: 200-tick memory on nonfatal in-water entity damage, ambient/enemy visibility suppression, and rehydrate/max-air constants
- [ ] Implement `Bee`: pollination flight from flower to hive, honey level increment (0–5), honeycomb drop on shear, sting → death, hive memory `MemoryModuleType.BEE_HIVE_REMEMBERED`, anger propagation
  - [x] Implement Java-matched `Bee` server-visible state gates: roll/stung/nectar flags, saved pollination/hive counters, 20-39s anger range, sting poison/death timing, underwater damage threshold, and hive-entry conditions
- [ ] Implement `Camel`: sitting/standing animation with transition ticks, dash ability cooldown, two-passenger riding with separate seat offsets
  - [x] Implement Java-matched `Camel` movement state: 55-tick dash cooldown, 5-tick minimum dash duration, sit/stand pose tick encoding, 40/52 tick pose-transition gates, sprint speed bonus, baby/sitting dimensions, and two-passenger attachment offsets
- [ ] Implement `Chicken`: egg-laying timer (5–10 minute interval), flutter-fall (no fall damage), baby chick → adult transition
  - [x] Implement Java-matched `Chicken` egg timer/flap fall model: 6000-11999 tick egg interval, non-baby non-jockey egg gate, falling Y velocity damping, baby dimensions, jockey despawn/XP behavior, and `EggLayTime` save field
- [ ] Implement `Cow`: mooshroom conversion via direct lightning strike, bucket milking interaction, breeding via wheat
  - [x] Implement Java-matched `Cow`/`AbstractCow`/`MushroomCow` interaction model: wheat food, adult bucket milking, baby dimensions, cow variant inheritance, mooshroom lightning toggle dedupe, and 1/1024 same-variant offspring mutation
- [ ] Implement `Dolphin`: locate-treasure AI (swim toward nearest buried treasure within 64 blocks), grace boost on nearby player swimming, strand and suffocate on land
  - [x] Implement Java-matched `Dolphin` server-visible gates: 4800 air, 2400 moistness, dry-out damage/jump sync, fish feeding `GotFish` or baby age-up, treasure goal air/radius/stop gates, and 100-tick Dolphin's Grace refresh
- [ ] Implement equine family: `AbstractHorse` (taming, `temper`, inventory, saddle, rider), `Horse` (armor slot, variants), `Donkey`/`Mule` (chest attachment), `ZombieHorse`, `SkeletonHorse` (trap activation), `Llama` (chest, carpet decoration, spit attack, caravan following), `TraderLlama`
- [ ] Implement feline family: `Cat` (gifts after player sleeps, biome-variant spawning, scared of players until tamed), `Ocelot` (chest-sitting prevention, trusts after fish feeding)
- [ ] Implement fish family: `Cod`, `Salmon`, `TropicalFish` (pattern/color variants, 2400 variant IDs), `Pufferfish` (inflation stages 0/1/2, poisonous contact at stage 2); all with `BucketableEntity`
  - [x] Implement Java-matched `Pufferfish` puff timing and contact effect model: states 0/1/2, inflate thresholds, deflate thresholds, `1 + puffState` damage, `60 * puffState` poison duration
  - [x] Implement Java-matched `Salmon` size variants: `small`/`medium`/`large` ids, default medium, bounding-box scales, and 30/50/15 spawn weights
  - [x] Implement Java-matched `TropicalFish` packed variant model: 12 pattern ids, base/pattern color bit layout, default KOB white/white, and 22 common variants
- [ ] Implement `Fox`: nocturnal chicken-hunting, item-holding/stealing, sweet-berry eating, trusting (bred from trusted parents), snow-dive pounce
- [ ] Implement `Frog`: tongue-attack on small slimes/magma cubes (absorb into stomach), tadpole-laying in water on breed, frog variant = biome (temperate/warm/cold)
- [ ] Implement `Goat`: ram-charge targeting (scream variant more frequent), horn-drop on successful ram, milking via bucket
  - [x] Implement Java-matched `Goat` server-visible gates: 2% screaming spawn chance, 10% adult missing-horn spawn chance, bucket milking gate, left/right horn-drop selection, ram/long-jump timing ranges, ram knockback, and lowered-head rotation
- [ ] Implement `IronGolem`: village-protection patrol AI, crack-stage visual from health percentage, pumpkin-carved face after player placement, rose-offer to villagers
- [ ] Implement `SnowGolem`: trail snow layer placement, melt in rain/warm biomes, pumpkin-carved face
- [ ] Implement `HappyGhast` (new in 26.1.2): large passive Nether mob, multiple-passenger riding (up to 4 harness slots), leads attachment, taming with dried ghast item, does not deal damage
- [x] Implement `Nautilus` variants (new variant types in 26.1.2 registry): `zombie_nautilus_variant` temperate/warm registry data, default save field, and warm metadata sync match Java `ZombieNautilusVariants`/`ZombieNautilus`
- [ ] Implement `Panda`: 7 personality traits (lazy/playful/worried/aggressive/weak/brown/normal), sneeze mechanic, rolling animation, bamboo eating
- [ ] Implement `Parrot`: imitation of nearby mob sounds, shoulder riding, dancing to jukebox within range, cookie-poisoning death
- [ ] Implement `Pig`: saddle+ride, carrot-on-a-stick steering and durability, lightning → ZombifiedPiglin conversion
  - [x] Implement Java-matched `Pig` riding and conversion gates: saddle-slot/adult checks, carrot-on-a-stick controller gate, 140-980 tick boost timer, ridden speed boost factor, 7-damage/25-durability boost item use, breeding variant inheritance, and non-peaceful lightning conversion
- [ ] Implement `PolarBear`: neutral until cub is nearby, aggressive to foxes, swim AI
  - [x] Implement Java-matched `PolarBear` server-visible gates: cub-protection player targeting, adult-only fox targeting and hurt alerts, standing warning attack window, 40-tick warning cooldown, 6-tick standing animation scale, 20-39s anger range, and 0.98 water slowdown
- [ ] Implement `Rabbit`: farmland/carrot-griefing AI, toast skin (username-triggered), killer bunny variant (only via `/summon`+NBT)
  - [x] Implement Java-matched `Rabbit` server-visible variant and garden gates: legacy ids 0/1/2/3/4/5/99, biome spawn weights, offspring variant inheritance, evil-rabbit attack/armor deltas, 40-tick carrot raid cooldown, carrot age decrement/destroy behavior, jump timing, baby dimensions, and non-evil avoid goals
- [ ] Implement `Sheep`: color-based wool drop, regrow wool after eating grass (`EatGrassGoal`), dyeing via dye item interaction
  - [x] Implement Java-matched `Sheep` server-visible wool model: packed color/sheared byte, 16-color legacy ids, shearing interaction gate and state change, grass-eating regrowth plus 60s baby age-up, 40-tick eat animation curves, biome spawn color weights including 1/500 pink common roll, and offspring dye-mix fallback
- [ ] Implement `Sniffer`: sniff animation, ancient-seed dropping at discovered location, digging-up animation, egg hatching, sniffing-exploration AI
- [ ] Implement `Squid`/`GlowSquid`: ink-squirt flee mechanic, glow squid dark-rendering effect, glowing while alive
  - [x] Implement Java-matched `Squid`/`GlowSquid` server-visible gates: 30-particle ink squirt only after mob-caused hurt, flee use within 10 blocks in water, flee vector speed taper 5-10 blocks with air Y clamp, bubble phase, baby dimensions, tentacle reset event, glow squid 100-tick darkening on hurt, dark tick decay/save field, and dark-water spawn checks
- [ ] Implement `Turtle`: beach-homing memory (home beach coordinates), egg-laying behavior, scute drop on growth, turtle-egg placement, egg-hatching process
- [ ] Implement `Wolf`: taming (bone-feed `temper`), collar color (default red, dyeable), wolf-armor equipping and display, pack-anger propagation on owner hit
- [ ] Add parity test for each animal's unique interaction: bee hive population, allay item pickup, sniffer dig timing, armadillo roll conditions

## NPC Families

- [ ] Implement `Villager` profession assignment: scans for POI workstations within range, acquires profession on first work block find, loses profession if work block removed
- [ ] Implement villager trade offers: load from `data/minecraft/villager_trade/<profession>.json` and `data/minecraft/trade_set/`, apply demand multiplier, hero-of-the-village discount
- [ ] Implement villager gossip system: `GossipType` (MAJOR_NEGATIVE/MINOR_NEGATIVE/TRADING/MAJOR_POSITIVE/MINOR_POSITIVE), gossip decay, propagation on villager meeting
- [ ] Implement villager schedule: `VillagerSchedules` (ADULT work/sleep/idle/meet, BABY play/sleep), `ActivitySchedule` time-of-day activity selection
- [ ] Implement villager restocking: 2 restock attempts per day at workstation, max 2 per day
- [ ] Implement villager breeding: willingness from trades + food, child spawning, child following parents
- [ ] Implement villager zombification and curing: weakness + golden apple conversion, cure-discount stack
- [ ] Implement `WanderingTrader`: spawn timer (24000-tick cycle), offer generation, lead-llamas attachment (2 trader llamas), despawn after 48000 ticks
- [ ] Implement `CatSpawner`: swamp hut (witch's hut) cat spawning, max 1 cat per hut
- [ ] Implement `WanderingTraderSpawner`: periodic spawn attempts near players (24h cycle)
- [ ] Add parity test: villager profession binding to specific POI type, trade restock timing, gossip propagation
- [ ] Add parity test: wandering trader spawn distance, llama count, despawn timer

## Raid Package

- [ ] Implement `Raid` lifecycle: omen acquisition → raid start (village detection), wave scheduling, wave-spawn positions, captain selection, hero-of-the-village reward on victory, raid defeat on village-capture
- [ ] Implement raid bossbar: purple color, health bar as fraction remaining-raids/total-waves
- [ ] Implement `PatrolSpawner`: 5-night patrol groups, captain selection, bad omen trigger on kill
- [ ] Implement raid wave composition by difficulty: wave sizes and mob types per wave per difficulty level
- [ ] Implement raid persistence: `raids/raids.dat` save/load, UUID tracking of active raids
- [ ] Implement `/raid` command hooks
- [ ] Add parity test: raid start conditions (≥3 villagers, omen carrier enters boundary), wave count per difficulty
- [ ] Add parity test: raid persistence through server restart, bossbar health fraction after wave wipe
- [ ] Add parity test: hero of the village effect duration and discount tier

## Entity Parity Tests (Cross-Cutting)

- [ ] For every entity family, add spawn-rules parity test: natural spawn conditions, light level, mob caps, biome restrictions
- [ ] For every entity family, add pathfinding parity test: target selection, obstacle avoidance, water/lava handling per type
- [ ] For every entity family, add interaction parity test: right-click tame/feed/milk/shear/saddle/ride/bucket
- [ ] For every entity family, add damage/drops parity test: death drops, XP table, looting enchantment multiplier
- [ ] For every entity family, add NBT save/load round-trip test: custom name, health, effects, inventory, AI flags
- [ ] For every entity family, add metadata-sync test: entity metadata packet field values match vanilla on state change
- [ ] For every entity family, add riding/passenger parity test: mounting, dismounting, passenger-offset sync
- [ ] For every entity family, add portal behavior test: nether portal teleport rules by entity type (mobs cannot teleport through end portal, etc.)
- [ ] For every entity family, add despawn test: natural despawn radius (32/128 blocks), named/persistent entities, chunk unload/reload
- [x] Add Mineflayer multi-bot entity-spawn visibility test: join two bots, verify mob-spawn packets match vanilla entity tracking range

## Migrated From Main Checklist: Entities And AI

- [ ] Implement base entity lifecycle: ID, UUID, type, position, rotation, velocity, bounding box, pose, flags, dimensions, passengers, vehicle, portal state, fire, air, freeze, fall, removal, save/load, and syncing.
- [ ] Implement all entity categories present under `net/minecraft/world/entity`.
- [ ] Implement living entity health, damage, armor, absorption, effects, attributes, equipment, hands, use item, death, drops, experience, knockback, and animation.
- [ ] Implement player entity server logic, abilities, game modes, hunger, saturation, exhaustion, experience, stats, advancements, recipe book, spawn, sleep, permissions, and interaction modes.
- [x] Add a Mineflayer player-state test that verifies health, food, saturation-visible effects, XP, game mode, permissions, and recipe book sync after offline-mode join.
- [x] Add raw 26.1.2 player-state join fallback coverage that verifies survival and creative offline profiles receive vanilla-shaped health, food, XP, game mode, abilities, and spawn position packets while Mineflayer lacks target-protocol play support.
- [x] Add player-entity recipe-book fallback coverage that persists and sync-plans known recipes plus open/filtering UI flags together with health, food, XP, permissions, and game-mode state while full Mineflayer join observation remains pending.
- [x] Add a Mineflayer offline-mode game-mode-persistence test that changes a bot between survival, creative, adventure, and spectator, reconnects each time, and verifies `force-gamemode` and saved game mode behavior match vanilla.
- [x] Add raw 26.1.2 game-mode persistence fallback coverage that seeds saved `playerGameType`/`previousPlayerGameType`, verifies fresh survival/creative/spectator profiles, numeric and invalid `gamemode` parsing, `force-gamemode=true` overrides, ability flags, and persisted effective game mode while Mineflayer lacks target-protocol play support.
- [x] Add a Mineflayer offline-mode respawn-after-relogin test that dies, disconnects on the death screen, reconnects with the same generated profile, and verifies vanilla-compatible death/respawn state recovery.
- [x] Add a Mineflayer death/respawn test that kills an offline-mode bot, verifies death message, respawn packet flow, inventory/XP rules, spawn position, and post-respawn abilities.
- [x] Add a Mineflayer multi-bot visibility test that joins two offline-mode bots, verifies tab-list entries, spawn/despawn packets, relative movement, sneaking/sprinting flags, held items, and disconnect cleanup.
- [x] Add a Mineflayer offline-mode entity-tracking distance test that moves bots across tracking thresholds and verifies spawn, metadata, velocity, equipment, and remove packets match vanilla timing.
- [ ] Implement item entities, experience orbs, falling blocks, TNT, end crystals, armor stands, paintings, item frames, leash knots, markers, interactions, displays if server-side relevant, and decorations.
- [ ] Implement projectiles: arrows, spectral arrows, tridents, snowballs, eggs, fireballs, wind charges, potions, ender pearls, fishing hooks, llama spit, shulker bullets, wither skulls, dragon fireballs, and thrown items.
- [ ] Implement vehicles: boats, chest boats, minecarts, furnace minecarts, chest minecarts, hopper minecarts, TNT minecarts, spawner minecarts, command block minecarts.
- [ ] Implement monsters, animals, ambient mobs, water mobs, NPCs, bosses, raids, and variants visible in the source tree.
- [ ] Implement AI goals, behavior trees, sensors, memories, activities, schedules, brain serialization, navigation, pathfinding, target selection, gossip, villager POIs, and village mechanics.
- [ ] Implement spawning rules, mob caps, despawn rules, natural spawning, patrols, wandering traders, phantoms, raids, trial spawners, monster rooms, chunk generation spawns, and command spawns.
- [ ] Implement boss fights: ender dragon, wither, bossbars, end crystals, gateways, dragon phases, and end fight state.
- [ ] Implement tameable, breedable, rideable, shearable, bucketable, variant, ageable, trading, anger, conversion, and transformation mechanics.
- [ ] Validate entity metadata and behavior with automated spawn/interact/kill/save/load tests.

## Migrated From Main Checklist: Source-Derived Granularity Appendix - Entity Family Coverage

- [ ] Implement shared AI packages: attributes, behavior, control, goal, gossip, memory, navigation, sensing, targeting, util, village, schedules, variants, and brain save/load.
- [ ] Implement ambient entities.
- [ ] Implement animal families: allay, armadillo, axolotl, bee, camel, chicken, cow, dolphin, equine, feline, fish, fox, frog, goat, golem, happy ghast, nautilus, panda, parrot, pig, polar bear, rabbit, sheep, sniffer, squid, turtle, and wolf.
- [ ] Implement boss families: ender dragon and wither, including phase/state machines, bossbars, crystals, summoning, death sequences, drops, and dimension-specific world state.
- [ ] Implement decoration entities: paintings, item frames, leash knots, armor stands, display-like entities if present, interaction entities, and marker behavior.
- [ ] Implement item entities and pickup/merge/despawn logic.
- [ ] Implement monster families: breeze, creaking, hoglin, illager, piglin, skeleton, spider, warden, zombie, and all shared hostile mob behavior.
- [ ] Implement NPC families: villagers, wandering traders, professions, trades, POIs, gossip, schedules, restocking, raids, and reputation.
- [ ] Implement player entity server behavior separately from generic living entity behavior.
- [ ] Implement projectile families: arrows, hurting projectiles, throwable item projectiles, owner tracking, collision, pierce, pickup, potion effects, loyalty, return, and despawn.
- [ ] Implement raid package behavior: raid lifecycle, waves, leaders, omen integration, hero rewards, persistence, bossbar, and village detection.
- [ ] Implement vehicles: boats, chest boats, minecarts, chest/hopper/furnace/TNT/spawner/command block minecarts, interpolation, collision, rails, activator/detector/powered rail behavior, and passenger sync.
- [ ] For every entity family, create parity scenarios for spawn rules, pathfinding, interactions, damage, drops, NBT save/load, metadata sync, riding/passengers, portals, despawn, and chunk unload/reload.
