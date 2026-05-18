# Game Protocol Packet Checklist

## Relevant Java Source Files

- `decompiled-server-26.1.2/net/minecraft/network/protocol/game/` — all 182 game-state packets
- `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundLoginPacket.java` — play login
- `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundRespawnPacket.java` — respawn
- `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundLevelChunkWithLightPacket.java` — chunk + light
- `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundPlayerPositionPacket.java` — position sync
- `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundMovePlayerPacket.java` — player movement
- `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetEntityDataPacket.java` — entity metadata
- `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundContainerSetContentPacket.java` — inventory sync
- `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundPlayerChatPacket.java` — signed chat
- `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundBossEventPacket.java` — bossbar
- `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundCommandsPacket.java` — command tree
- `decompiled-server-26.1.2/net/minecraft/network/codec/StreamCodec.java` — network codec base
- `RustCraft/src/network/play.rs` — RustCraft play state packet handling
- `RustCraft/src/network/protocol_coverage.rs` — RustCraft protocol coverage tracking

## Coverage Requirement

All 182 packets in `net/minecraft/network/protocol/game/` must be implemented. For each:
- Direction (clientbound or serverbound) and packet ID must match vanilla 26.1.2
- All fields must be serialized/deserialized in vanilla field order
- Optional fields and codec variants must match decompiled codec
- Malformed input must disconnect with vanilla-compatible reason

## Join / Respawn / World State Packets

- [ ] `ClientboundLoginPacket` (0x2B): entity ID, is-hardcore, game-type, previous-game-type, levels, registry-holder, dimension-type, dimension, seed, max-players, chunk-radius, simulation-distance, reduced-debug-info, enable-respawn-screen, do-limited-crafting, portal-cooldown, sea-level, enforces-secure-chat; verify all 19+ fields in exact order
- [ ] `ClientboundRespawnPacket` (0x45): common player spawn info, data-to-keep flags; exact flag bitmask for KEEP_ALL_DATA, KEEP_METADATA
- [ ] `ClientboundChangeDifficultyPacket` (0x0B): difficulty byte, difficulty-locked bool
- [ ] `ClientboundSetDefaultSpawnPositionPacket` (0x52): BlockPos, angle float
- [ ] `ClientboundSetTimePacket` (0x62): game-time long, day-time long, tick-day-time bool
- [ ] `ClientboundGameEventPacket` (0x22): event ID byte, param float (covers LEVEL_CHUNKS_LOAD_START, CHANGE_GAME_MODE, WIN_GAME, DEMO_EVENT, ARROW_HIT_PLAYER, RAIN_LEVEL_CHANGE, THUNDER_LEVEL_CHANGE, PUFFER_FISH_STING, GUARDIAN_ELDER_EFFECT, IMMEDIATE_RESPAWN)

## Chunk and Light Packets

- [ ] `ClientboundLevelChunkWithLightPacket` (0x27): chunk X/Z, chunk data (heightmaps, sections, block entities), light data (sky/block light set/empty bitsets, sky/block arrays, trust-edges flag); field order verified against decompiled codec
- [ ] `ClientboundForgetLevelChunkPacket` (0x1F): chunk X/Z
- [ ] `ClientboundLightUpdatePacket` (0x28): chunk X/Z, light data; separate from chunk-with-light for mid-game updates
- [ ] `ClientboundChunkBatchStartPacket` (0x0D): no fields
- [ ] `ClientboundChunkBatchFinishedPacket` (0x0C): batch-size int
- [ ] `ServerboundChunkBatchReceivedPacket` (0x09): desired-chunks-per-tick float; adaptive batching feedback
- [ ] `ClientboundSetChunkCacheCenterPacket` (0x50): chunk X/Z VarInts
- [ ] `ClientboundSetChunkCacheRadiusPacket` (0x51): view-distance VarInt
- [ ] `ClientboundSetSimulationDistancePacket` (0x61): simulation-distance VarInt
- [ ] `ClientboundMapItemDataPacket` (0x2D): map ID, scale, locked, optional tracking-position/decorations array, color patch or full color array; exact codec verified

## Entity Lifecycle Packets

- [ ] `ClientboundAddEntityPacket` (0x01): entity ID, UUID, type, pos X/Y/Z, pitch, yaw, head-yaw, data (varies by type), velocity X/Y/Z; verify type-specific `data` field encoding
- [ ] `ClientboundAddExperienceOrbPacket` (0x02): entity ID, position, count short
- [ ] `ClientboundRemoveEntitiesPacket` (0x40): VarInt array of entity IDs
- [ ] `ClientboundSetEntityMotionPacket` (0x57): entity ID, velocity X/Y/Z as shorts (units: 1/8000 blocks/tick)
- [ ] `ClientboundTeleportEntityPacket` (0x70): entity ID, pos X/Y/Z, velocity X/Y/Z, yaw/pitch bytes, on-ground bool
- [ ] `ClientboundRotateHeadPacket` (0x46): entity ID, head-yaw byte
- [ ] `ClientboundMoveEntityPacket.Pos` (0x2E): entity ID, delta X/Y/Z shorts, on-ground bool
- [ ] `ClientboundMoveEntityPacket.PosRot` (0x2F): entity ID, delta X/Y/Z, yaw/pitch, on-ground bool
- [ ] `ClientboundMoveEntityPacket.Rot` (0x30): entity ID, yaw/pitch, on-ground bool
- [ ] `ClientboundMoveVehiclePacket` (0x32): position, yaw, pitch
- [ ] `ClientboundSetPassengersPacket` (0x58): vehicle entity ID, passenger entity ID list
- [ ] `ClientboundEntityEventPacket` (0x1C): entity ID, event ID byte (living entity events: 2=hurt, 3=death, 6=tame-fail, 7=tame-success, etc.)

## Entity Metadata Packets

- [ ] `ClientboundSetEntityDataPacket` (0x54): entity ID, packed metadata list (index byte, type VarInt, value encoded by type)
- [ ] Implement all 26.1.2 entity metadata types: byte, varint, varlong, float, string, chat component, optional chat, item stack, boolean, rotations, block pos, optional block pos, direction, optional UUID, optional block state ID, optional global pos, nbt, particle, particle list, villager data, optional varint, pose, cat variant, wolf variant, frog variant, optional global position, painting variant, sniffer state, armadillo state, vector3f, quaternionf
- [ ] Implement entity metadata indexes for every entity type hierarchy (Entity, LivingEntity, Mob, PathfinderMob, Animal, Player, each monster/animal subtype)
- [ ] `ClientboundUpdateAttributesPacket` (0x72): entity ID, attribute list (ID, base value, modifier list with UUID/amount/operation)
- [ ] `ClientboundUpdateMobEffectPacket` (0x73): entity ID, effect ID VarInt, amplifier byte, duration VarInt, flags byte (ambient/visible/show-icon/blend)
- [ ] `ClientboundRemoveMobEffectPacket` (0x41): entity ID, effect ID VarInt
- [ ] `ClientboundAnimatePacket` (0x03): entity ID, animation byte (0=swing-main, 1=hurt, 2=wake-up, 3=swing-off, 4=critical, 5=magic-critical)
- [ ] `ClientboundSetEquipmentPacket` (0x59): entity ID, equipment list (slot+item pairs with `more` continuation flag)

## Inventory / Container Packets (Clientbound)

- [ ] `ClientboundContainerSetContentPacket` (0x12): window ID byte, state ID VarInt, items list, carried item
- [ ] `ClientboundContainerSetSlotPacket` (0x13): window ID byte, state ID VarInt, slot short, item
- [ ] `ClientboundContainerSetDataPacket` (0x11): window ID byte, property short, value short
- [ ] `ClientboundOpenScreenPacket` (0x35): window ID VarInt, menu type VarInt, title component
- [ ] `ClientboundContainerClosePacket` (0x10): window ID byte
- [ ] `ClientboundHorseScreenOpenPacket` (0x25): window ID byte, slot count VarInt, entity ID VarInt
- [ ] `ClientboundMerchantOffersPacket` (0x31): container ID, offers list (input1, input2 optional, result, uses, maxUses, xp, specialPrice, priceMultiplier, demand, ignoreDiscount), villager level, xp, is-regular-villager, can-restock
- [ ] `ClientboundSetCarriedItemPacket` (0x53): slot byte (0–8)
- [ ] `ClientboundCooldownPacket` (0x14): item VarInt, cooldown VarInt
- [ ] `ClientboundSetCursorItemPacket` (0x4F): cursor item

## Inventory / Container Packets (Serverbound)

- [ ] `ServerboundContainerClickPacket` (0x0C): window ID byte, state ID VarInt, slot short, button byte, click type VarInt, changed slots map, carried item
- [ ] `ServerboundContainerClosePacket` (0x0D): window ID byte
- [ ] `ServerboundSetCarriedItemPacket` (0x2B): slot short
- [ ] `ServerboundPickItemFromBlockPacket` and `ServerboundPickItemFromEntityPacket`: creative-mode middle-click
- [ ] `ServerboundEditBookPacket` (0x14): hand, pages list, optional title
- [ ] `ServerboundRenameItemPacket` (0x25): name string
- [ ] `ServerboundSelectTradePacket` (0x27): item number VarInt
- [ ] `ServerboundSetBeaconPacket` (0x28): primary effect optional, secondary effect optional
- [ ] `ServerboundSetCreativeModeSlotPacket` (0x2D): slot short, item
- [ ] `ServerboundContainerButtonClickPacket` (0x0B): window ID byte, button ID byte

## Recipe / Advancement / Unlock Packets

- [ ] `ClientboundRecipeBookAddPacket` (0x3C): recipe entries list (holder, display, forced-notification), replace flag
- [ ] `ClientboundRecipeBookRemovePacket` (0x3D): recipe IDs list
- [ ] `ClientboundRecipeBookSettingsPacket` (0x3E): recipe book open/filter flags per book type
- [ ] `ClientboundUpdateAdvancementsPacket` (0x74): reset/clear flag, added advancements map (ID → AdvancementHolder), removed advancement IDs, progress map (ID → criterion done-date map)
- [ ] `ServerboundRecipeBookChangeSettingsPacket` (0x21): book type, is-open, is-filter-active
- [ ] `ServerboundRecipeBookSeenRecipePacket` (0x22): recipe VarInt

## Commands / Suggestions Packets

- [ ] `ClientboundCommandsPacket` (0x0F): root node, full command tree with argument/literal/redirect nodes and permission flags
- [ ] `ClientboundCommandSuggestionsPacket` (0x10): transaction ID VarInt, range start/length, suggestions list (text, optional tooltip)
- [ ] `ServerboundCommandSuggestionPacket` (0x0A): transaction ID VarInt, command string

## Chat Packets

- [ ] `ClientboundPlayerChatPacket` (0x3A): sender UUID, index VarInt, optional signature, message body (plain/formatted), optional unsigned content, filter mask, chat type bound (type + sender name + optional target name)
- [ ] `ClientboundSystemChatPacket` (0x6C): content component, overlay bool
- [ ] `ClientboundDisguisedChatPacket` (0x19): content component, chat type bound
- [ ] `ClientboundDeleteChatPacket` (0x18): message signature bytes
- [ ] `ClientboundPlayerInfoUpdatePacket` (0x3D): action bitmask, entries list (UUID + per-action data: add-player name/properties, initialize-chat session, update-game-mode, update-listed, update-latency, update-display-name, update-hat, update-list-order)
- [ ] `ClientboundPlayerInfoRemovePacket` (0x3C): UUID list
- [ ] `ServerboundChatPacket` (0x06): message string, timestamp long, salt long, optional signature, last-seen messages (acknowledgment array + offset)
- [ ] `ServerboundChatCommandPacket` (0x04): command string, timestamp, salt, argument signatures, last-seen messages
- [ ] `ServerboundChatCommandSignedPacket` (0x05): same as signed variant
- [ ] `ServerboundChatSessionUpdatePacket` (0x07): profile public key (expires-at, key bytes, signature bytes)
- [ ] `ServerboundMessageAcknowledgementPacket` (0x03): offset int

## Scoreboard / Team Packets

- [ ] `ClientboundSetDisplayObjectivePacket` (0x55): slot byte, objective name string
- [ ] `ClientboundSetObjectivePacket` (0x56): objective name, mode byte (0=add, 1=remove, 2=change), optional display name/criteria/render-type/number-format
- [ ] `ClientboundSetScorePacket` (0x5A): owner string, objective name, score VarInt, optional display name, optional number format
- [ ] `ClientboundResetScorePacket` (0x42): owner string, optional objective name
- [ ] `ClientboundSetPlayerTeamPacket` (0x5B): team name, method byte (0=create, 1=remove, 2=update, 3=add-players, 4=remove-players), team data for create/update (display name, options bitmask, name-tag visibility, collision rule, color VarInt, prefix, suffix), player list for add/remove

## Bossbar Packet

- [ ] `ClientboundBossEventPacket` (0x0A): boss UUID, operation (add with name/progress/color/overlay/flags, remove, update-progress, update-name, update-style, update-flags)

## World Border Packets

- [ ] `ClientboundInitializeBorderPacket` (0x24): new center X/Z, old size, new size, lerp time, new absolute max size, warning blocks, warning time
- [ ] `ClientboundSetBorderCenterPacket` (0x4A): new center X/Z
- [ ] `ClientboundSetBorderLerpSizePacket` (0x4B): old size, new size, lerp time
- [ ] `ClientboundSetBorderSizePacket` (0x4C): new size
- [ ] `ClientboundSetBorderWarningDelayPacket` (0x4D): warning time
- [ ] `ClientboundSetBorderWarningDistancePacket` (0x4E): warning blocks

## Sound / Particle Packets

- [ ] `ClientboundSoundPacket` (0x65): sound holder (registered ID VarInt or inline sound event), source VarInt, pos X/Y/Z fixed-point, volume, pitch, seed long
- [ ] `ClientboundSoundEntityPacket` (0x66): same but entity ID instead of position
- [ ] `ClientboundStopSoundPacket` (0x67): optional source, optional sound name
- [ ] `ClientboundNamedSoundEffectPacket`: deprecated alias handled by `ClientboundSoundPacket`
- [ ] `ClientboundLevelParticlesPacket` (0x29): particle type VarInt, long distance bool, pos X/Y/Z, offset X/Y/Z, max speed, count, particle data (varies by type); verify all ~100 particle type data shapes
- [ ] `ClientboundLevelEventPacket` (0x28): event int, pos BlockPos, data int, global bool

## Map Packets

- [ ] `ClientboundMapItemDataPacket` (0x2D): map ID, scale byte, locked bool, optional decorations list (type/entity UUID/x/z/rot/optional label), optional patch (dirty x/z/width/height/colors bytes) or full 128×128 colors

## Title / Action Bar / Tab-List Packets

- [ ] `ClientboundSetTitleTextPacket` (0x60): title component
- [ ] `ClientboundSetSubtitleTextPacket` (0x5F): subtitle component
- [ ] `ClientboundSetTitlesAnimationPacket` (0x63): fade-in ticks, stay ticks, fade-out ticks
- [ ] `ClientboundClearTitlesPacket` (0x0E): reset bool
- [ ] `ClientboundSetActionBarTextPacket` (0x4D): action bar component
- [ ] `ClientboundTabListPacket` (0x6D): header component, footer component

## Player Abilities / Stats / Game Mode Packets

- [ ] `ClientboundPlayerAbilitiesPacket` (0x38): flags byte (invulnerable/flying/allow-flying/instant-build), flying speed, walking speed
- [ ] `ClientboundAwardStatsPacket` (0x05): stat map (stat type VarInt, stat ID VarInt → value VarInt)
- [ ] `ClientboundSetExperiencePacket` (0x56): experience-progress float, total-experience VarInt, level VarInt
- [ ] `ClientboundSetHealthPacket` (0x58): health float, food VarInt, saturation float
- [ ] `ClientboundGameEventPacket` (0x22): (also covers mode-change event 3 = change game mode)
- [ ] `ClientboundPlayerLookAtPacket` (0x39): from-anchor VarInt, target entity-or-block, optional entity-anchor

## Interactions / Block / Entity Actions (Serverbound)

- [ ] `ServerboundInteractPacket` (0x1A): entity ID, interaction type (attack/interact/interact-at with optional hand and hit position), using-secondary-action bool
- [ ] `ServerboundUseItemOnPacket` (0x36): hand, hit result (block pos, direction, hit vector, inside flag), sequence VarInt
- [ ] `ServerboundUseItemPacket` (0x37): hand, sequence VarInt, yaw/pitch
- [ ] `ServerboundPlayerActionPacket` (0x1D): action (start-dig, abort-dig, stop-dig, drop-all, drop-one, release-use, swap-held), block pos, face direction, sequence VarInt
- [ ] `ServerboundSwingPacket` (0x2E): hand VarInt
- [ ] `ServerboundPlayerCommandPacket` (0x1E): entity ID, action (start-sneaking, stop-sneaking, leave-bed, start-sprinting, stop-sprinting, start-riding-jump, stop-riding-jump, open-inventory, start-fall-flying), jump-boost int

## Player Movement (Serverbound)

- [ ] `ServerboundMovePlayerPacket.Pos` (0x1B): pos X/Y/Z, on-ground bool, horizontal-collision bool
- [ ] `ServerboundMovePlayerPacket.PosRot` (0x1C): pos X/Y/Z, yaw/pitch, on-ground bool, horizontal-collision bool
- [ ] `ServerboundMovePlayerPacket.Rot` (0x1D): yaw/pitch, on-ground bool, horizontal-collision bool
- [ ] `ServerboundMovePlayerPacket.StatusOnly` (0x1E): on-ground bool, horizontal-collision bool
- [ ] `ServerboundMoveVehiclePacket` (0x1F): pos X/Y/Z, yaw, pitch
- [ ] `ServerboundPaddleBoatPacket` (0x1E): left-paddle, right-paddle booleans
- [ ] `ServerboundPlayerInputPacket` (0x21): forward, backward, left, right floats, jump, shift, sprint flags (26.1.2 format)
- [ ] `ClientboundPlayerPositionPacket` (0x40): pos X/Y/Z, velocity X/Y/Z, yaw, pitch, relative flags (4-byte INT bitmask — not VarInt), teleport ID VarInt

## Block / Entity Action Packets (Clientbound)

- [ ] `ClientboundBlockUpdatePacket` (0x09): block pos, block state VarInt
- [ ] `ClientboundSectionBlocksUpdatePacket` (0x47): chunk section pos, block states array (packed pos+state longs)
- [ ] `ClientboundBlockEntityDataPacket` (0x07): block pos, type VarInt, NBT tag
- [ ] `ClientboundBlockEventPacket` (0x08): block pos, action byte, param byte, block type VarInt
- [ ] `ClientboundBlockDestructionPacket` (0x06): entity ID, block pos, progress byte (0–9, 10=done)
- [ ] `ClientboundExplodePacket` (0x1D): pos X/Y/Z, radius, affected blocks list (byte offsets), player velocity X/Y/Z, block interaction VarInt, small explosion particle, large explosion particle, sound
- [ ] `ClientboundEntityPositionSyncPacket` (0x20): (26.1.2 name for teleport ack; confirm ID VarInt)
- [ ] `ServerboundAcceptTeleportPacket` (0x00): teleport ID VarInt

## Resource State Packets

- [ ] `ClientboundResourcePackPopPacket`: resource pack UUID
- [ ] `ClientboundResourcePackPushPacket`: UUID, URL, hash, required, optional prompt component
- [ ] `ServerboundResourcePackPacket` (0x24): UUID, status VarInt (accepted/declined/failed/loaded)

## Debug / Misc Packets

- [ ] `ClientboundDebugSamplePacket` (0x17): sample array longs, sample type VarInt
- [ ] `ClientboundCustomPayloadPacket` (0x15) — `minecraft:brand` and other channels
- [ ] `ClientboundStartConfigurationPacket` (0x69): triggers switch from play back to configuration state
- [ ] `ServerboundConfigurationAcknowledgedPacket` (0x0E): triggers play→configuration ack
- [ ] `ClientboundPingPacket` (0x36): ID int
- [ ] `ServerboundPongPacket` (0x23): ID int
- [ ] `ClientboundDisconnectPacket` (0x1B): reason component
- [ ] `ServerboundSignUpdatePacket` (0x2C): block pos, is-front-text bool, 4 lines
- [ ] `ServerboundJigsawGeneratePacket` (0x1B): block pos, levels VarInt, keep-jigsaws bool
- [ ] `ServerboundSetStructureBlockPacket` (0x2F): block pos, update type, mode, offset, size, mirror, rotation, name, metadata, integrity, seed, flags
- [ ] `ServerboundSetCommandBlockPacket` (0x29): block pos, command, mode VarInt, flags byte
- [ ] `ServerboundSetCommandMinecartPacket` (0x2A): entity ID, command, track-output bool
- [ ] `ServerboundSetBeaconPacket` (0x28): primary effect optional VarInt, secondary effect optional VarInt

## Protocol Coverage Requirements

- [ ] For each of the 182 game-state packets: record packet ID, direction, field order, codec, optional fields, registry dependencies, version gates, compression behavior, and disconnect behavior for malformed input
- [ ] For every clientbound packet: add a golden serialization test generated from a vanilla server traffic capture
- [ ] For every serverbound packet: add a decoder fuzz test and a replay test confirming vanilla-compatible server-side effects
