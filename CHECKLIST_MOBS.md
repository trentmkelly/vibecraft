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
- [ ] Implement `WitherBoss` three-head targeting (distinct target per head), skull projectile alternation, Java head launch offsets/direction, main-head 0.1% blue-skull roll, and level event 1024 on non-silent launch
- [ ] Implement `WitherSkull` projectile: normal/charged dangerous state, Java inertia 0.95 vs 0.73, save/load `dangerous`, owner/magic damage, owner heal-on-kill, Normal/Hard wither effect durations, power-1.0 explosion, and charged resistance cap for destructible blocks
  - [ ] Verify Java edge case: charged wither skulls do not increase explosion power; `WitherSkull.getBlockExplosionResistance()` caps destructible block resistance at `min(0.8, resistance)` while `onHit()` still explodes at power 1.0
- [ ] Implement Java-matched Wither powered target movement below half health: `isPowered()` changes vertical chase gating while main-target horizontal acceleration/yaw follow `WitherBoss.aiStep`; Java 26.1.2 does not define a separate charge-attack phase
- [ ] Implement wither shield/invulnerability below half health (Java powered gate rejects arrows and wind charges; other non-immune damage still passes normal gates)
- [ ] Implement wither self-healing over time and on skull hit
- [ ] Implement wither "wither effect" aura on hit
- [ ] Implement wither death/drop sequence: Java death loot drops an extended-lifetime nether star; the 7.0 explosion belongs to the spawn invulnerability completion, and bossbar visibility ends with entity/player tracking removal
- [ ] Implement wither summoning precondition check (wither skeleton skull item, Y >= minY+2, non-peaceful server level, T-shape soul sand/soil base + 3 wither skulls/wall skulls, both X/Z orientations; Java 26.1.2 `WitherSkullBlock.canSpawnMob` has no superflat-End gate)
- [ ] Implement wither bossbar: `BossEvent.BossBarColor.PURPLE`, health %
- [ ] Implement wither block-breaking behavior (breaks most blocks in path)
- [ ] Add parity test: wither skull target assignment, blue skull vs. normal skull NBT
- [ ] Add parity test: shield phase invulnerability threshold and self-heal rate

## Monster Families

### Top-Level Monsters
- [ ] Implement `Blaze`: fire charge burst (3 fireballs per attack cycle), hover flight AI, fire immunity, drop (blaze rod)
- [ ] Implement `Creeper`: fuse countdown (`fuseTime`), explosion on fuse complete, `ignited` NBT (flint+steel trigger), charged variant (lightning conversion), `ExplosionPower` scaling for charged
  - [ ] Implement Java-matched `Creeper` server-visible state gates: default `Fuse=30`/radius 3/save fields, igniter item consume-vs-damage behavior, `ignited` forcing swell dir 1, primed sound/game event on first positive swell tick, fall-distance swell preload capped at `maxSwell - 5`, explosion on max swell, powered lightning state with 2x radius, charged-creeper loot one-shot gate, active-effect lingering cloud constants, goat target exclusion, and swelling interpolation denominator.
- [ ] Implement `ElderGuardian`: elder curse (Mining Fatigue III, 6000 ticks) within 50 blocks, beam laser at aquatic player targets, spike thorns passive
- [ ] Implement `EnderMan`: holding block (pickup/place logic), teleport on water/rain/projectiles, look-at-eyes anger trigger, stare sound/scream state on anger
- [ ] Implement `Endermite`: spawn from ender pearl (5% chance), 2-minute despawn timer, attacked by endermen
- [ ] Implement `Ghast`: attack cycle (charge fireball, shoot), large hitbox (4×4×4), fire immunity, scream/shoot sounds, deflectable fireball by projectiles/swords
- [ ] Implement `Giant`: legacy oversized zombie (no natural spawn, only `/summon`), 12-block height, no AI goals
- [ ] Implement `Guardian`: laser beam targeting (beam charge, inflict damage at full charge), spikes thorns, water spawn/pathing gates
- [ ] Implement `MagmaCube`: split into smaller cubes on death (size-1 → 2–4 smaller), slime-family movement, fire immunity
  - [ ] Implement Java-matched `MagmaCube` server-visible slime-family gates: peaceful-only spawn rejection, inherited size clamp/split offsets/counts, armor = size×3, create-attributes base speed 0.2 plus inherited size-scaled movement, always-effective-AI damage even when tiny, attack damage +2, 4x jump delay, 0.9 squish decay, `isOnFire() == false`, ground jump +0.1×size, and lava jump `0.22 + 0.05×size`.
- [ ] Implement `Phantom`: insomnia prerequisite spawn (no sleep >3 days), circling-swoop attack, Java-matched no-daylight-burn behavior, phantom membrane drop, and direct nearby-player targeting rather than `MemoryModuleType.NEAREST_PLAYERS`
  - [ ] Implement Java-matched `Phantom` server-visible spawn and attack gates: `PhantomSpawner` insomnia timing/group/position offsets, size clamp/save/default anchor, attack damage and dimension scaling by size, flap tick cadence, target scan range/cadence, circle-to-swoop delay, anchor reselection, cat-aborted swoops, collision/hurt cancellation, level event 1039 on non-silent sweep hits, no daylight burn hook in `Phantom.java`, no `MemoryModuleType.NEAREST_PLAYERS`, and killed-by-player phantom membrane loot.
- [ ] Implement `Ravager`: attack windup, shield-block stun/knockback, roar AoE damage/knockback, and Java-matched leaf-only mob-griefing path behavior
- [ ] Implement `Shulker`: attached face (6 directions), shell open/close armor/peek state, homing `ShulkerBullet` projectile, teleport/reproduction gates on damage, levitation effect on hit, color variants
- [ ] Implement `Silverfish`: merge into compatible stone/cobblestone/brick/deepslate host blocks, infested-block break spawns, emergency-call behavior waking nearby infested blocks
- [ ] Implement `Slime`: size variants (1/2/4), split on death, water/lava float movement, harmless when `size <= 1`
  - [ ] Implement Java-matched `Slime` server-visible size and split gates: size clamp 1..127, natural spawn sizes 1/2/4, save `Size=size-1` plus `wasOnGround`, max-health `size²`, movement speed `0.2 + 0.1×size`, attack damage/xp = size, tiny harmless gate, 2-4 half-size death split children with Java offsets, squish/landing particle timing, spawn surface/swamp and slime-chunk gates, jump-delay/aggressive-delay math, float-goal/move-control gates, and passenger attachment/sound-volume constants.
- [ ] Implement `Strider`: lava walking, saddle+ride with warped-fungus-on-a-stick steering, suffocating cold-block state, attraction to warped fungus, jockey spawn rolls
- [ ] Implement `Vex`: summoned by evoker (3 per cast, nearby-count gate), phase through solid blocks, 30–119 second limited life after summon
- [ ] Implement `Witch`: splash potion attack (harm/slowness/weakness/poison selection by context), self-healing with instant-health/fire-resistance/water-breathing/swiftness potions, raid healer targeting; note 26.1.2 raid Ravager riders are not witches
- [ ] Implement `Zoglin`: permanent hostile target filtering, Hoglin conversion into Zoglin after 300 Overworld/End ticks, HoglinBase knockback attack, no regular zombification state

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
- [ ] Implement `Hoglin`: adult/baby attack gates, hunting eligibility, repellent/piglin avoidance, spawn/walk targeting, and conversion state
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
  - [ ] Implement Java-matched `AbstractPiglin` conversion gates: default pickup/immune/time-in-overworld save values, `PIGLINS_ZOMBIFY && !NoAI && !IsImmuneToZombification`, increment/reset `TimeInOverworld`, convert after `> 300` ticks to zombified piglin with `ConversionParams.single(..., keepEquipment=true, preserveCanPickUpLoot=true)`, 200-tick nausea, and Piglin override behavior that cancels admiring and drops inventory before conversion
- [ ] Implement `Piglin`: bartering mechanic (gold ingot throw → loot drop from `data/minecraft/loot_table/gameplay/piglin_bartering.json`), admire-item behavior, anger-on-gold-theft, crossbow attack, dance-to-jukebox memory, baby piglin
- [ ] Implement `PiglinAi` full behavior tree: `StartAdmiringItemIfSeen`, `StopAdmiringIfItemTooFarAway`, `StopAdmiringIfTiredOfTryingToReachItem`, `StopHoldingItemIfNoLongerAdmiring`, `RememberIfHoglinWasKilled`, `StartHuntingHoglin`
- [ ] Implement `PiglinBrute`: no bartering, enhanced attack, always hostile to players (never passive)
  - [ ] Implement Java-matched `PiglinBrute` gates: 50 health, 0.35 movement speed, 7 attack damage, 12 follow range, 20 XP, guaranteed golden axe main hand, `canHunt=false`, only picks up golden axes if base pickup accepts them, anger memory 600 ticks, melee attack cooldown 20 ticks, idle/fight activity constants, targeting priority `ANGRY_AT` then nearest visible attackable player then nearest visible nemesis, no retaliation against other abstract piglins, and melee attacking arm pose
- [ ] Implement `PiglinBruteAi` brain behavior tree
- [ ] Add parity test: piglin bartering loot table output distribution across gold throws
- [ ] Add parity test: piglin anger triggers (opening gold chest, picking up gold in front of piglin)

### Skeleton Family
- [ ] Implement `AbstractSkeleton`: Java attack interval selection, bow-vs-melee goal gate, ranged arrow speed/inaccuracy/lead constants, pickup loot chance, Halloween head selection, and shared entity dimensions/sounds
- [ ] Implement `Skeleton`: powder-snow conversion to Stray at 140 ticks plus 300-tick conversion countdown/save value
- [ ] Implement `Stray`: slowness-tipped arrows, powder-snow immunity, and sky/spawner spawn gate
- [ ] Implement `WitherSkeleton`: fire/wither-rose immunity, wither-effect sword melee, flaming arrows, wither immunity, lava pathfinding malus, and wither dimensions
- [ ] Implement `Bogged`: poison-tipped arrows, 16-health variant attributes, sheared state, and shearing gate
- [ ] Implement `Parched`: weakness-tipped arrows, 16-health variant attributes, weakness immunity, and non-fire-immune Java surface behavior
- [ ] Add parity test: skeleton attack interval/arrow accuracy, stray powder-snow conversion tick count, and skeleton variant effects/immunities

### Spider Family
- [ ] Implement `Spider`: wall-climbing behavior via `WallClimberNavigation` and climbing flag on horizontal collision, spider-jockey (1% chance with skeleton mount), neutral in daylight, hostile in dark
  - [ ] Implement Java-matched shared `Spider` server-visible gates used by cave spiders: climbing flag bit, attack disabled while mounted as vehicle, dark-only target acquisition, 1% skeleton-jockey finalize-spawn gate, hard-difficulty special effect roll/table, armadillo-avoidance gate, bright-light target drop gate, poison immunity marker, and vehicle attachment constants.
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
- [ ] Implement `Zombie`: drowning conversion to Drowned (baby cannot convert), baby zombie (5% chance, speed/XP/dimensions), reinforcement summoning on hit (hard + `doMobSpawning`), villager conversion chance, fire-on-hit, loot pickup, door breaking, default equipment, chicken jockeys, Halloween headgear, and save-field gates
- [ ] Implement `Drowned`: trident attack AI, swim AI, water/depth spawn gates, nautilus-shell offhand chance, zombie-nautilus jockey chance, spear pickup rejection
- [ ] Implement `Husk`: desert biome only, does not burn in daylight, hunger effect on hit, water conversion to Zombie, baby dimensions, loot pickup chance, camel-husk/parched rider finalize-spawn gate
- [ ] Implement `ZombieVillager`: Weakness + golden apple starts 3600-6000 tick cure, special-block speedup, conversion to Villager preserving villager data/gossips/offers/XP, cured-villager reputation event for downstream discounts, baby dimensions, despawn gate, and offer reset on profession change
- [ ] Implement `ZombifiedPiglin`: neutral persistent anger (20-39s), first anger sound delay, alert-other propagation over follow range + 10Y, attacking speed modifier, spawn/obstruction rules, default golden sword/spear equipment, rest prevention, and Nether portal random-spawn gate
- [ ] Add parity test: zombie villager cure duration and resulting price discount values
- [ ] Add parity test: zombified piglin anger group radius and anger decay time

## Animal Families

- [ ] Implement `Allay`: follow note-block memory, collect items matching held item, duplicate on jukebox play + amethyst shard, dancing-to-jukebox memory
  - [ ] Implement Java-matched `Allay` server-visible gates: 20 health/0.1 flying and movement speed/2 attack attributes, 1x1x1 pickup reach, one-slot inventory, liked-player immunity/ally behavior, liked-player distance/game-mode gate, item give/take interactions and sounds, mobGriefing-gated wanted-item pickup with potion-content equality, noteblock vibration memory and 600 tick cooldown, noteblock/player deposit-target selection, jukebox play/stop dancing memory and stop conditions, dancing/spinning animation timing, amethyst-shard duplication with 6000 tick parent/child cooldowns and event 18/3 hearts, 10-tick passive healing cadence, no far-away despawn, leash offset, and throw-sound timing gate
- [ ] Implement `Armadillo`: roll into ball on threat (player sprint/mount approach), scute dropping, wolf-armor crafting ingredient source
  - [ ] Implement Java-matched `Armadillo` server-visible gates: idle/rolling/scared/unrolling state ids, names, animation durations and shell-hide thresholds, 12 health/0.14 speed attributes, 0.6 baby scale, spider-eye food, grass/badlands/red-sand/coarse-dirt spawn blocks, 6000-11999 tick scute shed timer, shed/brush loot tables and sounds, 16 durability brush cost, scare detection from undead/last-hurt-by/sprinting-or-riding non-spectator players inside 7x2x7 box, roll-up/out sounds and state transitions, scared damage reduction, 80 tick danger memory, environmental roll-out, fall-in-love and ambient/hurt/head-rotation gates, and peek/unroll ball-up timing
- [ ] Implement `Axolotl`: play-dead behavior (`DeathAnimation`), attack aquatic hostiles (guardians, drowned, etc.), `BucketableEntity` bucket capture, axolotl color variants (5 types including rare blue)
  - [ ] Implement Java-matched `Axolotl` color variant model: lucy/wild/gold/cyan common variants, rare blue variant, default lucy, and 1/1200 rare breeding chance
  - [ ] Implement Java-matched `Axolotl` play-dead gates: 200-tick memory on nonfatal in-water entity damage, ambient/enemy visibility suppression, and rehydrate/max-air constants
  - [ ] Implement Java-matched `Axolotl` bucket/attack/support gates: water-bucket pickup, bucket save fields, from-bucket persistence/despawn rule, hunt target tags and cooldown, always-hostile aquatic targets, assisted-kill player regeneration/mining-fatigue removal, attributes, and baby dimensions
- [ ] Implement `Bee`: pollination flight from flower to hive, honey level increment (0–5), honeycomb drop on shear, sting → death, hive memory `MemoryModuleType.BEE_HIVE_REMEMBERED`, anger propagation
  - [ ] Implement Java-matched `Bee` server-visible state gates: roll/stung/nectar flags, saved pollination/hive counters, 20-39s anger range, sting poison/death timing, underwater damage threshold, and hive-entry conditions
  - [ ] Implement Java-matched `Bee` hive/block gates: 3-bee occupancy cap, 600/2400 tick hive stays by nectar state, honey delivery level increment capped at 5 with 1% +2 roll, crop-growth cap/chance, honeycomb/shear and bottle reset interactions, smoke sedation anger suppression, emergency/front-blocked release behavior, and saved flower transfer
- [ ] Implement `Camel`: sitting/standing animation with transition ticks, dash ability cooldown, two-passenger riding with separate seat offsets
  - [ ] Implement Java-matched `Camel` movement state: 55-tick dash cooldown, 5-tick minimum dash duration, sit/stand pose tick encoding, 40/52 tick pose-transition gates, sprint speed bonus, baby/sitting dimensions, and two-passenger attachment offsets
- [ ] Implement `Chicken`: egg-laying timer (5–10 minute interval), flutter-fall (no fall damage), baby chick → adult transition
  - [ ] Implement Java-matched `Chicken` egg timer/flap fall model: 6000-11999 tick egg interval, non-baby non-jockey egg gate, falling Y velocity damping, baby dimensions, jockey despawn/XP behavior, and `EggLayTime` save field
- [ ] Implement `Cow`: mooshroom conversion via direct lightning strike, bucket milking interaction, breeding via wheat
  - [ ] Implement Java-matched `Cow`/`AbstractCow`/`MushroomCow` interaction model: wheat food, adult bucket milking, baby dimensions, cow variant inheritance, mooshroom lightning toggle dedupe, and 1/1024 same-variant offspring mutation
- [ ] Implement `Dolphin`: locate-treasure AI (swim toward nearest buried treasure within 64 blocks), grace boost on nearby player swimming, strand and suffocate on land
  - [ ] Implement Java-matched `Dolphin` server-visible gates: 4800 air, 2400 moistness, dry-out damage/jump sync, fish feeding `GotFish` or baby age-up, treasure goal air/radius/stop gates, and 100-tick Dolphin's Grace refresh
- [ ] Implement equine family: `AbstractHorse` (taming, `temper`, inventory, saddle, rider), `Horse` (armor slot, variants), `Donkey`/`Mule` (chest attachment), `ZombieHorse`, `SkeletonHorse` (trap activation), `Llama` (chest, carpet decoration, spit attack, caravan following), `TraderLlama`
  - [ ] Implement Java-matched equine server-visible gates: `AbstractHorse` flags/slot offsets/temper clamp, horse variant+marking packed metadata and breeding inheritance rolls, chested-horse chest inventory columns/equip gate, llama strength/variant clamping, chest columns, spit attack constants, max temper, and breeding strength/variant rolls
- [ ] Implement feline family: `Cat` (gifts after player sleeps, biome-variant spawning, scared of players until tamed), `Ocelot` (chest-sitting prevention, trusts after fish feeding)
  - [ ] Implement Java-matched feline server-visible gates: shared 10 health/0.3 movement/3 attack attributes, crouch/walk/sprint move-control pose mapping, cod/salmon food tags, cat default black variant plus all-black structure/moon spawn priority, red default collar and dye interactions, cat taming/heal/toggle-sit interaction flow, untamed player avoidance and despawn timing, bed/chest/furnace sitting target rules, owner-sleep relax and morning-gift gates, stray cat spawner village/hut caps, ocelot trust feeding event gates, ocelot player avoidance/despawn/spawn obstruction, and ocelot leash offset
- [ ] Implement fish family: `Cod`, `Salmon`, `TropicalFish` (pattern/color variants, 2400 variant IDs), `Pufferfish` (inflation stages 0/1/2, poisonous contact at stage 2); all with `BucketableEntity`
  - [ ] Implement Java-matched `AbstractFish`/`AbstractSchoolingFish` gates: 3 health, from-bucket persistence/despawn rules, water-bucket pickup surface, flop jump/sync/sound, water travel/no-target gravity, avoid-player/swim goal constants, max cluster size 8, schooling follower/leader limits, neighbor reset scan, Cod/Salmon/TropicalFish/Pufferfish bucket items, and Salmon max school size 5
  - [ ] Implement Java-matched `Pufferfish` puff timing and contact effect model: states 0/1/2, inflate thresholds, deflate thresholds, `1 + puffState` damage, `60 * puffState` poison duration
  - [ ] Implement Java-matched `Salmon` size variants: `small`/`medium`/`large` ids, default medium, bounding-box scales, and 30/50/15 spawn weights
  - [ ] Implement Java-matched `TropicalFish` packed variant model: 12 pattern ids, base/pattern color bit layout, default KOB white/white, and 22 common variants
- [ ] Implement `Fox`: nocturnal chicken-hunting, item-holding/stealing, sweet-berry eating, trusting (bred from trusted parents), snow-dive pounce
  - [ ] Implement Java-matched `Fox` server-visible gates: red/snow biome variant ids, sitting/crouching/interested/pouncing/sleeping/faceplanted/defending flags, trusted offspring slots from breeding players, sweet/glow berry food and item replacement rules, mobGriefing-gated berry harvest wait/counts, and chicken/rabbit stalking state transition gates
- [ ] Implement `Frog`: Java-matched biome-tag variant selection (warm/cold with temperate fallback), frog-food gate for size-1 slimes and magma cubes, tongue catch/eat timing and pose/sounds, and breeding `IS_PREGNANT`/`LAY_SPAWN` frogspawn placement plan
- [ ] Implement `Goat`: ram-charge targeting (scream variant more frequent), horn-drop on successful ram, milking via bucket
  - [ ] Implement Java-matched `Goat` server-visible gates: 2% screaming spawn chance, 10% adult missing-horn spawn chance, bucket milking gate, left/right horn-drop selection, ram/long-jump timing ranges, ram knockback, and lowered-head rotation
- [ ] Implement `IronGolem`: village-protection patrol AI, crack-stage visual from health percentage, pumpkin-carved face after player placement, rose-offer to villagers
  - [ ] Implement Java-matched `IronGolem` server-visible gates: player-created metadata/save flag from carved pumpkin/jack-o-lantern block summoning, player-created/player and creeper attack exclusions, 100 health/0.25 speed/1.0 knockback resistance/15 attack damage attributes, crackiness thresholds at 75%/50%/25%, 400-tick offer-flower event ids 11/34, 10-tick attack event id 4, iron-ingot repair for up to 25 health, and vertical attack knockback scaled by target knockback resistance
- [ ] Implement `SnowGolem`: Java-matched pumpkin flag/save state, shearing gate/drop/tool damage, warm/rain environment melt damage, mobGriefing-gated four-offset snow trail placement, and snowball attack vector/speed/inaccuracy
- [ ] Implement `HappyGhast` (new in 26.1.2): large passive Nether mob, multiple-passenger riding (up to 4 harness slots), leads attachment, taming with dried ghast item, does not deal damage
  - [ ] Correct Java parity edge case: Happy Ghasts are not tamed with a dried ghast item; `DriedGhastBlock` water hydration increments to level 3, then removes the block and spawns a baby Happy Ghast with the ghastling spawn sound
  - [ ] Implement Java-matched `HappyGhast` server-visible gates: 20 health/16 tempt range/0.05 movement and flying speed/16 follow range/8 camera distance attributes, 0.2375 baby scale, snowball food, 16 harness items, adult-only body harness slot, harness-gated riding, max 4 passengers, harness/still-timeout/player first-passenger control gate, 64/32 restriction radius selection, 20/600 tick continuous heal intervals, still-timeout decay and player-above reset, collision gates, quad-leash offsets/support, 10/16 leash distances, and 5-tick leash-holder notification
- [ ] Implement `Nautilus` variants (new variant types in 26.1.2 registry): `zombie_nautilus_variant` temperate/warm registry data, default save field, and warm metadata sync match Java `ZombieNautilusVariants`/`ZombieNautilus`
- [ ] Implement `Panda`: 7 personality traits (lazy/playful/worried/aggressive/weak/brown/normal), sneeze mechanic, rolling animation, bamboo eating
  - [ ] Implement Java-matched `Panda` server-visible gates: seven-gene id/name/recessive model and random weights, variant derivation from main/hidden genes, sneeze/roll/sit/on-back flags, weak/lazy attribute overrides, bamboo/cake food gates, bamboo interaction outcomes, 32-step roll movement timing, and sneeze sound/particle/loot lifecycle
- [ ] Implement `Parrot`: imitation of nearby mob sounds, shoulder riding, dancing to jukebox within range, cookie-poisoning death
  - [ ] Implement Java-matched `Parrot` server-visible gates: five variant ids/names with clamped legacy decode, 6 health/0.4 flying speed/0.2 movement/3 attack attributes, seed tag taming with 1-in-10 event ids 7/6, cookie poison for 900 ticks plus lethal damage unless invulnerable, owner sitting toggle only while grounded, jukebox dance invalidation at 3.46 blocks or missing jukebox, 1-in-400 AI mimic attempt and 1-in-2 mimic sound gate over the vanilla hostile sound map, and shoulder riding cooldown/owner/player acceptance gates
- [ ] Implement `Pig`: saddle+ride, carrot-on-a-stick steering and durability, lightning → ZombifiedPiglin conversion
  - [ ] Implement Java-matched `Pig` riding and conversion gates: saddle-slot/adult checks, carrot-on-a-stick controller gate, 140-980 tick boost timer, ridden speed boost factor, 7-damage/25-durability boost item use, breeding variant inheritance, and non-peaceful lightning conversion
- [ ] Implement `PolarBear`: neutral until cub is nearby, aggressive to foxes, swim AI
  - [ ] Implement Java-matched `PolarBear` server-visible gates: cub-protection player targeting, adult-only fox targeting and hurt alerts, standing warning attack window, 40-tick warning cooldown, 6-tick standing animation scale, 20-39s anger range, and 0.98 water slowdown
- [ ] Implement `Rabbit`: farmland/carrot-griefing AI, toast skin (username-triggered), killer bunny variant (only via `/summon`+NBT)
  - [ ] Implement Java-matched `Rabbit` server-visible variant and garden gates: legacy ids 0/1/2/3/4/5/99, biome spawn weights, offspring variant inheritance, evil-rabbit attack/armor deltas, 40-tick carrot raid cooldown, carrot age decrement/destroy behavior, jump timing, baby dimensions, and non-evil avoid goals
- [ ] Implement `Sheep`: color-based wool drop, regrow wool after eating grass (`EatGrassGoal`), dyeing via dye item interaction
  - [ ] Implement Java-matched `Sheep` server-visible wool model: packed color/sheared byte, 16-color legacy ids, shearing interaction gate and state change, grass-eating regrowth plus 60s baby age-up, 40-tick eat animation curves, biome spawn color weights including 1/500 pink common roll, and offspring dye-mix fallback
- [ ] Implement `Sniffer`: sniff animation, ancient-seed dropping at discovered location, digging-up animation, egg hatching, sniffing-exploration AI
  - [ ] Implement Java-matched `Sniffer` server-visible gates: state ids 0-6 with zero fallback, 14 health/0.1 speed attributes, -48000 baby age, torchflower seed food, diggable block/hatch boost tags, can-sniff/can-dig body gates, transition sounds/events including digging event 63 and seed drop at tick+120, digging particle/seed/game-event timing, 20 remembered explored-position cap, mating states, breeding sniffer egg drop, 9600 tick sniff cooldown timing, 160-180/40/600 tick dig/search behavior constants, and sniffer egg 24000/12000 hatch schedule with crack/hatch behavior
- [ ] Implement `Squid`/`GlowSquid`: ink-squirt flee mechanic, glow squid dark-rendering effect, glowing while alive
  - [ ] Implement Java-matched `Squid`/`GlowSquid` server-visible gates: 30-particle ink squirt only after mob-caused hurt, flee use within 10 blocks in water, flee vector speed taper 5-10 blocks with air Y clamp, bubble phase, baby dimensions, tentacle reset event, glow squid 100-tick darkening on hurt, dark tick decay/save field, and dark-water spawn checks
- [ ] Implement `Turtle`: beach-homing memory (home beach coordinates), egg-laying behavior, scute drop on growth, turtle-egg placement, egg-hatching process
  - [ ] Implement Java-matched `Turtle` server-visible gates: home-pos save/default model, has-egg/laying-egg flags and counter reset, 30 health/0.25 movement/1.0 step attributes, 0.3 baby scale, seagrass food, breeding sets egg instead of spawning child plus parent ages/xp/stat trigger, egg-laying target/home gates and 200-tick placement delay with 1-4 eggs, 5-tick digging particle event, scute gift loot on adult growth, spawn-on-sand/sea-level gate, go-home/go-water/travel gates, water sinking rule, no leash, lethal lightning damage, and turtle egg crush/crack/hatch/replacement behavior
- [ ] Implement `Wolf`: taming (bone-feed `temper`), collar color (default red, dyeable), wolf-armor equipping and display, pack-anger propagation on owner hit
  - [ ] Implement Java-matched `Wolf` server-visible interaction gates: untamed bone consume with 1-in-3 tame success, tame side effects to 40 max health and ordered sitting, default red owner-only collar dyeing, owner-only adult body armor equip, sitting armor repair by armadillo scute at 12.5% max durability, meat healing, and owner-hurt/owner-target anger assignment.
- [ ] Add parity test for each animal's unique interaction: bee hive population, allay item pickup, sniffer dig timing, armadillo roll conditions
  - [ ] Add focused Java parity tests for bee hive population, sniffer dig timing, and armadillo roll conditions
  - [ ] Add focused Java parity test for Allay item pickup, note-block deposit memory, jukebox dancing, and amethyst duplication gates

## NPC Families

- [ ] Implement `Villager` profession assignment: scans for POI workstations within range, acquires profession on first work block find, loses profession if work block removed
  - [ ] Implement Java-matched villager profession/job-site gates: default plains/none/level-1 data, level XP thresholds 0/10/70/150/250, workstation-to-profession mapping, held/acquirable job-site predicates, work sounds, offer clearing on profession change, job-site ticket acquire/release intent, stop-trading when profession becomes none, breeding spawn profession reset, structure spawn assign-profession flag, and profession loss when remembered job-site no longer matches
- [ ] Implement villager trade offers: load from `data/minecraft/villager_trade/<profession>.json` and `data/minecraft/trade_set/`, apply demand multiplier, hero-of-the-village discount
  - [ ] Implement Java-matched trade resource and pricing gates: decode `VillagerTrade` JSON defaults for wants/additional_wants/gives/max_uses/reputation_discount/xp/predicates/modifiers/double-price enchantments, decode `TradeSet` holder sets/random sequences, apply demand multiplier in `MerchantOffer.getModifiedCostCount`, apply reputation discounts via `-floor(reputation * priceMultiplier)`, apply Hero of the Village discount `floor((0.3 + 0.0625 * amplifier) * baseCostA)` with minimum 1, and reset special prices when trading stops
- [ ] Implement villager gossip system: `GossipType` (MAJOR_NEGATIVE/MINOR_NEGATIVE/TRADING/MAJOR_POSITIVE/MINOR_POSITIVE), gossip decay, propagation on villager meeting
  - [ ] Implement Java-matched villager gossip gates: five gossip types with serialized names, weights, max values, daily decay, transfer decay, discard threshold 2, capped additions, daily decay removal below threshold, deterministic selected-gossip transfer with max(old, transferred) merge, villager meeting transfer gate at 1200 ticks for both participants, 24000 tick gossip decay cadence, and reputation event mappings for cured/trade/hurt/killed events
- [ ] Implement villager schedule: `VillagerSchedules` (ADULT work/sleep/idle/meet, BABY play/sleep), `ActivitySchedule` time-of-day activity selection
  - [ ] Implement Java 26.1.2 timeline-backed villager activity gates: adult `VILLAGER_ACTIVITY` keyframes at 10 idle / 2000 work / 9000 meet / 11000 idle / 12000 rest, baby `BABY_VILLAGER_ACTIVITY` keyframes at 10 idle / 3000 play / 6000 idle / 10000 play / 12000 rest, 24000-tick periodic wrap, age-based schedule selection, and Brain schedule update cadence of `gameTime - lastScheduleUpdate > 20`
- [ ] Implement villager restocking: 2 restock attempts per day at workstation, max 2 per day
  - [ ] Implement Java-matched villager restock gates: always reports `canRestock`, restock updates demand/resets uses/resends offers, records `lastRestockGameTime`, increments `numberOfRestocksToday`, allows first restock or second only after `> 2400` ticks, resets daily restock count after `lastRestock + 12000` or day-period advance, requires at least one offer needing restock, and catch-up demand applies `2 - restocksToday` demand updates with use reset when positive
- [ ] Implement villager breeding: willingness from trades + food, child spawning, child following parents
  - [ ] Implement Java-matched villager breeding/food gates: bread=4 and potato/carrot/beetroot=1 food points, 12-point breed/hunger threshold, 24-point excess threshold, sleep/adult breed blockers, eat-until-full inventory consumption order, 12-point digestion, pickup gate for villager food/plantable seeds plus farmer requested items, inventory-capacity gate, and offspring villager type selection with 50% biome / 25% first parent / 25% second parent weights
- [ ] Implement villager zombification and curing: weakness + golden apple conversion, cure-discount stack
  - [ ] Covered by Java-matched zombie-villager and villager trade tests: Weakness + golden apple interaction, 3600-6000 tick cure timer, conversion tick/progress with iron-bars/bed acceleration, villager data/gossips/offers/XP preservation, cured-player advancement/reputation event gate, 200-tick nausea, level event 1027, despawn gate, and cured-villager major/minor positive gossip discounts
- [ ] Implement `WanderingTrader`: spawn timer (24000-tick cycle), offer generation, lead-llamas attachment (2 trader llamas), despawn after 48000 ticks
- [ ] Implement `CatSpawner`: swamp hut (witch's hut) cat spawning, max 1 cat per hut
  - [ ] Implement Java-matched `CatSpawner` server-visible gates as part of feline slice: 1200 tick delay, player-relative 8-31 block offset, 10-block chunk-ready check surface, village home/cat caps, swamp-hut one-cat cap, and persistent hut cat spawn flag
- [ ] Implement `WanderingTraderSpawner`: periodic spawn attempts near players (24h cycle)
  - [ ] Implement Java-matched `WanderingTraderSpawner` server-visible gates: 1200 tick custom-spawner cadence, saved spawn delay decrement by 1200, 24000 reset, 25/75 spawn-chance clamp with 25 increments, `nextInt(100) <= chance` outer roll, no-player attempt success behavior, 1-in-10 spawn gate, meeting-POI/player reference radius 48, 10 candidate positions inside +/-48, 12-block collision-space check, biome exclusion gate, two trader llamas within radius 4, trader despawn delay 48000, home radius 16, and no far-away despawn
- [ ] Add parity test: villager profession binding to specific POI type, trade restock timing, gossip propagation
  - [ ] Add focused Java parity test for villager profession binding to workstation POIs, profession change/loss behavior, level XP thresholds, and existing restock/gossip trade model
- [ ] Add parity test: wandering trader spawn distance, llama count, despawn timer
  - [ ] Add focused Java parity test for wandering trader spawner timing, spawn chance, candidate offsets, llama count, and despawn countdown

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
- [ ] Add Mineflayer multi-bot entity-spawn visibility test: join two bots, verify mob-spawn packets match vanilla entity tracking range

## Migrated From Main Checklist: Entities And AI

- [ ] Implement base entity lifecycle: ID, UUID, type, position, rotation, velocity, bounding box, pose, flags, dimensions, passengers, vehicle, portal state, fire, air, freeze, fall, removal, save/load, and syncing.
- [ ] Implement all entity categories present under `net/minecraft/world/entity`.
- [ ] Implement living entity health, damage, armor, absorption, effects, attributes, equipment, hands, use item, death, drops, experience, knockback, and animation.
- [ ] Implement player entity server logic, abilities, game modes, hunger, saturation, exhaustion, experience, stats, advancements, recipe book, spawn, sleep, permissions, and interaction modes.
- [ ] Add a Mineflayer player-state test that verifies health, food, saturation-visible effects, XP, game mode, permissions, and recipe book sync after offline-mode join.
- [ ] Add raw 26.1.2 player-state join fallback coverage that verifies survival and creative offline profiles receive vanilla-shaped health, food, XP, game mode, abilities, and spawn position packets while Mineflayer lacks target-protocol play support.
- [ ] Add player-entity recipe-book fallback coverage that persists and sync-plans known recipes plus open/filtering UI flags together with health, food, XP, permissions, and game-mode state while full Mineflayer join observation remains pending.
- [ ] Add a Mineflayer offline-mode game-mode-persistence test that changes a bot between survival, creative, adventure, and spectator, reconnects each time, and verifies `force-gamemode` and saved game mode behavior match vanilla.
- [ ] Add raw 26.1.2 game-mode persistence fallback coverage that seeds saved `playerGameType`/`previousPlayerGameType`, verifies fresh survival/creative/spectator profiles, numeric and invalid `gamemode` parsing, `force-gamemode=true` overrides, ability flags, and persisted effective game mode while Mineflayer lacks target-protocol play support.
- [ ] Add a Mineflayer offline-mode respawn-after-relogin test that dies, disconnects on the death screen, reconnects with the same generated profile, and verifies vanilla-compatible death/respawn state recovery.
- [ ] Add a Mineflayer death/respawn test that kills an offline-mode bot, verifies death message, respawn packet flow, inventory/XP rules, spawn position, and post-respawn abilities.
- [ ] Add a Mineflayer multi-bot visibility test that joins two offline-mode bots, verifies tab-list entries, spawn/despawn packets, relative movement, sneaking/sprinting flags, held items, and disconnect cleanup.
- [ ] Add a Mineflayer offline-mode entity-tracking distance test that moves bots across tracking thresholds and verifies spawn, metadata, velocity, equipment, and remove packets match vanilla timing.
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
