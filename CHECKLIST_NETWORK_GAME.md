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

- [x] `ClientboundLoginPacket` (0x2B): entity ID, is-hardcore, levels, max-players, chunk-radius, simulation-distance, reduced-debug-info, enable-respawn-screen, do-limited-crafting, common player spawn info (dimension type, dimension, seed, game-type, previous-game-type, debug/flat flags, last death location, portal cooldown, sea level), enforces-secure-chat; verified field order against Java
- [x] `ClientboundRespawnPacket` (0x45): common player spawn info, data-to-keep flags; exact flag bitmask for KEEP_ALL_DATA, KEEP_ATTRIBUTE_MODIFIERS, KEEP_ENTITY_DATA
- [x] `ClientboundChangeDifficultyPacket` (0x0B): difficulty byte, difficulty-locked bool
- [x] `ClientboundSetDefaultSpawnPositionPacket` (0x52): BlockPos, angle float
- [x] `ClientboundSetTimePacket` (0x62): game-time long, day-time long, tick-day-time bool
- [x] `ClientboundGameEventPacket` (0x22): event ID byte, param float (covers LEVEL_CHUNKS_LOAD_START, CHANGE_GAME_MODE, WIN_GAME, DEMO_EVENT, ARROW_HIT_PLAYER, RAIN_LEVEL_CHANGE, THUNDER_LEVEL_CHANGE, PUFFER_FISH_STING, GUARDIAN_ELDER_EFFECT, IMMEDIATE_RESPAWN)

## Chunk and Light Packets

- [x] `ClientboundLevelChunkWithLightPacket` (0x27): fixed-int chunk X/Z, chunk data (heightmaps map, section buffer, block entities), light data (sky/block light set/empty bitsets, sky/block arrays); field order verified against decompiled codec
- [x] `ClientboundForgetLevelChunkPacket` (0x1F): chunk X/Z
- [x] `ClientboundLightUpdatePacket` (0x28): VarInt chunk X/Z, light data; separate from chunk-with-light for mid-game updates
- [x] `ClientboundChunkBatchStartPacket` (0x0D): no fields
- [x] `ClientboundChunkBatchFinishedPacket` (0x0C): batch-size int
- [x] `ServerboundChunkBatchReceivedPacket` (0x09): desired-chunks-per-tick float; adaptive batching feedback
- [x] `ClientboundSetChunkCacheCenterPacket` (0x50): chunk X/Z VarInts
- [x] `ClientboundSetChunkCacheRadiusPacket` (0x51): view-distance VarInt
- [x] `ClientboundSetSimulationDistancePacket` (0x61): simulation-distance VarInt
- [x] `ClientboundMapItemDataPacket` (0x2D): map ID, scale, locked, optional tracking-position/decorations array, color patch or full color array; exact codec verified

## Entity Lifecycle Packets

- [x] `ClientboundAddEntityPacket` (0x01): entity ID, UUID, type, pos X/Y/Z, pitch, yaw, head-yaw, data (varies by type), velocity X/Y/Z; verify type-specific `data` field encoding
- [x] `ClientboundAddExperienceOrbPacket`: not present in 26.1.2 Java `GamePacketTypes`; experience orbs spawn through `ClientboundAddEntityPacket`
- [x] `ClientboundRemoveEntitiesPacket` (0x4D): VarInt entity ID list (`FriendlyByteBuf.writeIntIdList`)
- [x] `ClientboundSetEntityMotionPacket` (0x65): entity ID VarInt, velocity as `Vec3.LP_STREAM_CODEC` with vanilla clamping
- [x] `ClientboundTeleportEntityPacket` (0x7D): entity ID, `PositionMoveRotation` (position Vec3, delta movement Vec3, yaw/pitch floats), relative flags int bitmask, on-ground bool
- [x] `ClientboundRotateHeadPacket` (0x53): entity ID VarInt, head-yaw byte
- [x] `ClientboundMoveEntityPacket.Pos` (0x2E): entity ID VarInt, delta X/Y/Z shorts, on-ground bool
- [x] `ClientboundMoveEntityPacket.PosRot` (0x2F): entity ID VarInt, delta X/Y/Z shorts, yaw/pitch bytes, on-ground bool
- [x] `ClientboundMoveEntityPacket.Rot` (0x30): entity ID VarInt, yaw/pitch bytes, on-ground bool
- [x] `ClientboundMoveVehiclePacket` (0x39): position `Vec3.STREAM_CODEC`, yaw float, pitch float
- [x] `ClientboundSetPassengersPacket` (0x6B): vehicle entity ID VarInt, passenger entity ID VarInt array
- [x] `ClientboundEntityEventPacket` (0x22): entity ID int, event ID byte (living entity events: 2=hurt, 3=death, 6=tame-fail, 7=tame-success, etc.)

## Entity Metadata Packets

- [x] `ClientboundSetEntityDataPacket` (0x54): entity ID, packed metadata list (index byte, type VarInt, value encoded by type)
- [x] Implement all 26.1.2 entity metadata types: byte, varint, varlong, float, string, chat component, optional chat, item stack, boolean, rotations, block pos, optional block pos, direction, optional UUID, optional block state ID, optional global pos, nbt, particle, particle list, villager data, optional varint, pose, cat variant, wolf variant, frog variant, optional global position, painting variant, sniffer state, armadillo state, vector3f, quaternionf
- [x] Implement entity metadata indexes for every entity type hierarchy (Entity, LivingEntity, Mob, PathfinderMob, Animal, Player, each monster/animal subtype)
- [x] `ClientboundUpdateAttributesPacket` (0x72): entity ID, attribute registry holder ID, base value, modifier list with identifier/amount/operation
- [x] `ClientboundUpdateMobEffectPacket` (0x84): entity ID VarInt, effect registry ID VarInt, amplifier VarInt, duration VarInt, flags byte (ambient/visible/show-icon/blend)
- [x] `ClientboundRemoveMobEffectPacket` (0x41): entity ID, effect ID VarInt
- [x] `ClientboundAnimatePacket` (0x02): entity ID VarInt, animation unsigned byte (0=swing-main, 2=wake-up, 3=swing-off, 4=critical, 5=magic-critical)
- [x] `ClientboundSetEquipmentPacket` (0x59): entity ID, equipment list (slot+item pairs with `more` continuation flag)

## Inventory / Container Packets (Clientbound)

- [x] `ClientboundContainerSetContentPacket` (0x12): window ID byte, state ID VarInt, items list, carried item
- [x] `ClientboundContainerSetSlotPacket` (0x13): window ID byte, state ID VarInt, slot short, item
- [x] `ClientboundContainerSetDataPacket` (0x13): container ID VarInt, property short, value short
- [x] `ClientboundOpenScreenPacket` (0x35): window ID VarInt, menu type VarInt, title component
- [x] `ClientboundContainerClosePacket` (0x11): container ID VarInt
- [x] `ClientboundMountScreenOpenPacket` (0x29, replaces older HorseScreenOpen): container ID VarInt, inventory columns VarInt, entity ID fixed int
- [x] `ClientboundMerchantOffersPacket` (0x31): container ID, offers list (input1 `ItemCost`, result `ItemStack`, optional input2 `ItemCost`, out-of-stock, uses, maxUses, xp, specialPrice, priceMultiplier, demand), villager level, xp, show-progress, can-restock
- [x] `ClientboundSetHeldSlotPacket` (0x69): slot VarInt (renamed from older carried item packet)
- [x] `ClientboundCooldownPacket` (0x16): cooldown group Identifier, cooldown duration VarInt
- [x] `ClientboundSetCursorItemPacket` (0x4F): cursor item

## Inventory / Container Packets (Serverbound)

- [x] `ServerboundContainerClickPacket` (0x12): container ID VarInt, state ID VarInt, slot short, button byte, container input VarInt id mapper, changed slots map capped at 128 entries, carried `HashedStack`
- [x] `ServerboundContainerClosePacket` (0x13): container ID VarInt
- [x] `ServerboundSetCarriedItemPacket` (0x35): slot short
- [x] `ServerboundPickItemFromBlockPacket` (0x24): block pos, include-data bool; `ServerboundPickItemFromEntityPacket` (0x25): entity ID VarInt, include-data bool
- [x] `ServerboundEditBookPacket` (0x18): slot VarInt, pages list capped at 100 entries with 1024-char UTF-8 pages, optional 32-char title
- [x] `ServerboundRenameItemPacket` (0x30): name UTF-8 string capped at 32767 chars
- [x] `ServerboundSelectTradePacket` (0x33): item number VarInt
- [x] `ServerboundSetBeaconPacket` (0x34): primary effect optional MobEffect registry id, secondary effect optional MobEffect registry id
- [x] `ServerboundSetCreativeModeSlotPacket` (0x38): slot short, `ItemStack.OPTIONAL_UNTRUSTED_STREAM_CODEC` with count VarInt, optional item registry ID, and delimited data component patch framing
- [x] `ServerboundContainerButtonClickPacket` (0x11): container ID VarInt, button ID VarInt

## Recipe / Advancement / Unlock Packets

- [x] `ClientboundRecipeBookAddPacket` (0x3C): recipe entries list (display entry, notification/highlight flags), replace flag
- [x] `ClientboundRecipeBookRemovePacket` (0x3D): recipe IDs list
- [x] `ClientboundRecipeBookSettingsPacket` (0x3E): recipe book open/filter flags per book type
- [x] `ClientboundUpdateAdvancementsPacket` (0x74): reset/clear flag, added advancement holders, removed advancement IDs, progress map, show-advancements flag
- [x] `ServerboundRecipeBookChangeSettingsPacket` (0x2E): book type enum VarInt, is-open bool, is-filter-active bool
- [x] `ServerboundRecipeBookSeenRecipePacket` (0x2F): recipe display ID VarInt index

## Commands / Suggestions Packets

- [x] `ClientboundCommandsPacket` (0x0F): root node, full command tree with argument/literal/redirect nodes and executable/restricted flags
- [x] `ClientboundCommandSuggestionsPacket` (0x10): transaction ID VarInt, range start/length, suggestions list (text, optional tooltip component via trusted network NBT tag)
- [x] `ServerboundCommandSuggestionPacket` (0x0F): transaction ID VarInt, command UTF-8 string capped at 32500 chars

## Chat Packets

- [x] `ClientboundPlayerChatPacket` (0x3A): global index, sender UUID, message index, optional signature, packed signed body, optional unsigned content, filter mask, chat type bound (type + sender name + optional target name)
- [x] `ClientboundSystemChatPacket` (0x6C): content component via `ComponentSerialization.TRUSTED_STREAM_CODEC` network NBT tag, overlay bool
- [x] `ClientboundDisguisedChatPacket` (0x19): content component, chat type bound with registered chat type holder, sender name, optional target name
- [x] `ClientboundDeleteChatPacket` (0x1F): packed message signature (`VarInt(id + 1)` or full 256-byte signature)
- [x] `ClientboundPlayerInfoUpdatePacket` (0x3D): action fixed-bitset, entries list (UUID + per-action data: add-player name/properties, initialize-chat session, update-game-mode, update-listed, update-latency, update-display-name, update-list-order, update-hat)
- [x] `ClientboundPlayerInfoRemovePacket` (0x45): UUID list
- [x] `ServerboundChatPacket` (0x09): message string max 256, timestamp epoch millis long, salt long, optional 256-byte signature, last-seen update (offset VarInt + fixed 20-bit acknowledgment bitset + checksum byte)
- [x] `ServerboundChatCommandPacket` (0x07): command string max 32767
- [x] `ServerboundChatCommandSignedPacket` (0x08): command string max 32767, timestamp epoch millis long, salt long, argument signatures capped at 8 entries with 16-char names and 256-byte signatures, last-seen update
- [x] `ServerboundChatSessionUpdatePacket` (0x0A): chat session UUID, profile public key data (expires-at epoch millis, public key byte array capped at 512 bytes, signature byte array capped at 4096 bytes)
- [x] `ServerboundChatAckPacket` (0x06): message acknowledgement offset VarInt

## Scoreboard / Team Packets

- [x] `ClientboundSetDisplayObjectivePacket` (0x55): slot byte, objective name string
- [x] `ClientboundSetObjectivePacket` (0x56): objective name, mode byte (0=add, 1=remove, 2=change), optional display name/render-type/number-format
- [x] `ClientboundSetScorePacket` (0x5A): owner string, objective name, score VarInt, optional display name, optional number format
- [x] `ClientboundResetScorePacket` (0x42): owner string, optional objective name
- [x] `ClientboundSetPlayerTeamPacket` (0x5B): team name, method byte (0=create, 1=remove, 2=update, 3=add-players, 4=remove-players), team data for create/update (display name, options bitmask, name-tag visibility, collision rule, color VarInt, prefix, suffix), player list for add/remove

## Bossbar Packet

- [x] `ClientboundBossEventPacket` (0x0A): boss UUID, operation (add with name/progress/color/overlay/flags, remove, update-progress, update-name, update-style, update-flags)

## World Border Packets

- [x] `ClientboundInitializeBorderPacket` (0x2B): new center X/Z doubles, old size double, new size double, lerp time VarLong, new absolute max size VarInt, warning blocks VarInt, warning time VarInt
- [x] `ClientboundSetBorderCenterPacket` (0x58): new center X/Z doubles
- [x] `ClientboundSetBorderLerpSizePacket` (0x59): old size double, new size double, lerp time VarLong
- [x] `ClientboundSetBorderSizePacket` (0x5A): new size double
- [x] `ClientboundSetBorderWarningDelayPacket` (0x5B): warning time VarInt
- [x] `ClientboundSetBorderWarningDistancePacket` (0x5C): warning blocks VarInt

## Sound / Particle Packets

- [x] `ClientboundSoundPacket` (0x65): sound holder (registered ID + 1 or inline sound event), source VarInt, pos X/Y/Z fixed-point ints, volume, pitch, seed long
- [x] `ClientboundSoundEntityPacket` (0x66): sound holder (registered ID + 1 or inline sound event), source VarInt, entity ID VarInt, volume, pitch, seed long
- [x] `ClientboundStopSoundPacket` (0x77): flags byte, optional sound source enum, optional sound identifier
- [x] `ClientboundNamedSoundEffectPacket`: not present in 26.1.2 Java `GamePacketTypes`; handled by `ClientboundSoundPacket`
- [x] `ClientboundLevelParticlesPacket` (0x29): override-limiter bool, always-show bool, pos X/Y/Z, offset X/Y/Z, max speed, count, particle registry type VarInt plus particle-specific payload
- [x] `ClientboundLevelEventPacket` (0x2E): event int, pos BlockPos, data int, global bool

## Map Packets

- [x] `ClientboundMapItemDataPacket` (0x2D): map ID, scale byte, locked bool, optional decorations list (type/x/y/rot/optional label), optional patch (dirty x/y/width/height/colors bytes) or full 128x128 colors

## Title / Action Bar / Tab-List Packets

- [x] `ClientboundSetTitleTextPacket` (0x60): title component via `ComponentSerialization.TRUSTED_STREAM_CODEC` network NBT tag
- [x] `ClientboundSetSubtitleTextPacket` (0x5F): subtitle component via `ComponentSerialization.TRUSTED_STREAM_CODEC` network NBT tag
- [x] `ClientboundSetTitlesAnimationPacket` (0x73): fade-in ticks int, stay ticks int, fade-out ticks int
- [x] `ClientboundClearTitlesPacket` (0x0E): reset-times bool
- [x] `ClientboundSetActionBarTextPacket` (0x4D): action bar component via `ComponentSerialization.TRUSTED_STREAM_CODEC` network NBT tag
- [x] `ClientboundTabListPacket` (0x6D): header component, footer component via `ComponentSerialization.TRUSTED_STREAM_CODEC` network NBT tags

## Player Abilities / Stats / Game Mode Packets

- [x] `ClientboundPlayerAbilitiesPacket` (0x40): flags byte (invulnerable/flying/allow-flying/instant-build), flying speed float, walking speed float
- [x] `ClientboundAwardStatsPacket` (0x05): stat map (stat type VarInt, stat ID VarInt -> value VarInt)
- [x] `ClientboundSetExperiencePacket` (0x56): experience-progress float, total-experience VarInt, level VarInt
- [x] `ClientboundSetHealthPacket` (0x58): health float, food VarInt, saturation float
- [x] `ClientboundGameEventPacket` (0x26): event unsigned byte and float parameter (also covers mode-change event 3 = change game mode)
- [x] `ClientboundPlayerLookAtPacket` (0x47): from-anchor enum VarInt, target X/Y/Z doubles, optional entity ID VarInt and to-anchor enum VarInt

## Interactions / Block / Entity Actions (Serverbound)

- [x] `ServerboundChangeDifficultyPacket` (0x04): difficulty enum byte
- [x] `ServerboundClientCommandPacket` (0x0C): action enum VarInt (`PERFORM_RESPAWN`, `REQUEST_STATS`, `REQUEST_GAMERULE_VALUES`)
- [x] `ServerboundClientTickEndPacket` (0x0D): empty payload
- [x] `ServerboundLockDifficultyPacket` (0x1D): locked bool
- [x] `ServerboundInteractPacket` (0x1A): entity ID VarInt, interaction hand enum VarInt, low-precision Vec3 location, using-secondary-action bool
- [x] `ServerboundUseItemOnPacket` (0x42): hand enum VarInt, block hit result (block pos, direction enum VarInt, hit vector floats, inside flag, world-border-hit flag), sequence VarInt
- [x] `ServerboundUseItemPacket` (0x43): hand enum VarInt, sequence VarInt, yaw/pitch floats
- [x] `ServerboundPlayerActionPacket` (0x29): action enum VarInt (start-destroy, abort-destroy, stop-destroy, drop-all, drop-one, release-use, swap-offhand, stab), block pos, face direction byte, sequence VarInt
- [x] `ServerboundSwingPacket` (0x3F): hand VarInt
- [x] `ServerboundPlayerCommandPacket` (0x2A): entity ID VarInt, action enum VarInt (stop-sleeping, start-sprinting, stop-sprinting, start-riding-jump, stop-riding-jump, open-inventory, start-fall-flying), data VarInt

## Player Movement (Serverbound)

- [x] `ServerboundMovePlayerPacket.Pos` (0x1E): pos X/Y/Z, on-ground bool, horizontal-collision bool
- [x] `ServerboundMovePlayerPacket.PosRot` (0x1F): pos X/Y/Z, yaw/pitch, on-ground bool, horizontal-collision bool
- [x] `ServerboundMovePlayerPacket.Rot` (0x20): yaw/pitch, on-ground bool, horizontal-collision bool
- [x] `ServerboundMovePlayerPacket.StatusOnly` (0x21): on-ground bool, horizontal-collision bool
- [x] `ServerboundMoveVehiclePacket` (0x22): pos X/Y/Z, yaw, pitch, on-ground bool
- [x] `ServerboundPaddleBoatPacket` (0x23): left-paddle, right-paddle booleans
- [x] `ServerboundPlayerInputPacket` (0x2B): single-byte `Input.STREAM_CODEC` bitset for forward, backward, left, right, jump, shift, and sprint flags (26.1.2 format)
- [x] `ServerboundPlayerLoadedPacket` (0x2C): empty payload
- [x] `ClientboundPlayerPositionPacket` (0x48): id VarInt, `PositionMoveRotation` (position Vec3, delta movement Vec3, yaw/pitch floats), relative flags fixed 4-byte INT bitmask

## Block / Entity Action Packets (Clientbound)

- [x] `ClientboundBlockUpdatePacket` (0x08): block pos, block state VarInt
- [x] `ClientboundSectionBlocksUpdatePacket` (0x47): section pos long, VarInt count, packed VarLong entries (`block_state_id << 12 | packed_section_pos`)
- [x] `ClientboundBlockEntityDataPacket` (0x07): block pos, block entity type registry VarInt, trusted compound NBT tag
- [x] `ClientboundBlockEventPacket` (0x07): block pos, action unsigned byte, param unsigned byte, block type VarInt
- [x] `ClientboundBlockDestructionPacket` (0x05): entity ID VarInt, block pos, progress unsigned byte (0-9, 10=done)
- [x] `ClientboundExplodePacket` (0x1D): center Vec3, radius float, block count int, optional player knockback Vec3, explosion particle, sound holder, weighted block explosion particles
- [x] `ClientboundEntityPositionSyncPacket` (0x23): entity ID VarInt, `PositionMoveRotation` (position Vec3, delta movement Vec3, yaw/pitch floats), on-ground bool
- [x] `ServerboundAcceptTeleportationPacket` (0x00): teleport ID VarInt

## Resource State Packets

- [x] `ClientboundResourcePackPopPacket`: optional resource pack UUID
- [x] `ClientboundResourcePackPushPacket`: UUID, URL, hash (max 40), required, optional prompt component via trusted context-free network NBT tag
- [x] `ServerboundResourcePackPacket` (0x31): UUID, action enum VarInt (`SUCCESSFULLY_LOADED`, `DECLINED`, `FAILED_DOWNLOAD`, `ACCEPTED`, `DOWNLOADED`, `INVALID_URL`, `FAILED_RELOAD`, `DISCARDED`)

## Debug / Misc Packets

- [x] `ClientboundDebugSamplePacket` (0x17): sample array longs, sample type VarInt
- [x] `ClientboundCustomPayloadPacket` (0x18) — `minecraft:brand` and unknown payload channels up to vanilla 1 MiB cap
- [x] `ClientboundStartConfigurationPacket` (0x69): triggers switch from play back to configuration state
- [x] `ServerboundConfigurationAcknowledgedPacket` (0x10): empty terminal payload; triggers play→configuration ack
- [x] `ClientboundPingPacket` (0x3D): ID int
- [x] `ServerboundPongPacket` (0x2D): ID int
- [x] `ClientboundDisconnectPacket` (0x20 in 26.1.2 play/common): reason component via `ComponentSerialization.TRUSTED_CONTEXT_FREE_STREAM_CODEC` network NBT tag
- [x] `ServerboundSignUpdatePacket` (0x3D): block pos, is-front-text bool, 4 UTF-8 lines capped at 384 chars each
- [x] `ServerboundJigsawGeneratePacket` (0x1B): block pos, levels VarInt, keep-jigsaws bool
- [x] `ServerboundSetStructureBlockPacket` (0x3B): block pos, update type enum, mode enum, name, clamped offset/size bytes, mirror/rotation enums, metadata string capped at 128 chars, clamped integrity, seed VarLong, flags byte (ignore-entities=1, show-air=2, show-bounding-box=4, strict=8)
- [x] `ServerboundSetCommandBlockPacket` (0x36): block pos, command UTF-8 string, mode enum VarInt, flags byte (track-output=1, conditional=2, automatic=4)
- [x] `ServerboundSetCommandMinecartPacket` (0x37): entity ID VarInt, command UTF-8 string, track-output bool
- [x] `ServerboundSetBeaconPacket` (0x34): primary effect optional MobEffect registry id, secondary effect optional MobEffect registry id

## Protocol Coverage Requirements

- [ ] For each of the 182 game-state packets: record packet ID, direction, field order, codec, optional fields, registry dependencies, version gates, compression behavior, and disconnect behavior for malformed input
- [ ] For every clientbound packet: add a golden serialization test generated from a vanilla server traffic capture
- [ ] For every serverbound packet: add a decoder fuzz test and a replay test confirming vanilla-compatible server-side effects

## Migrated From Main Checklist: Network Transport

- [x] Implement TCP listener on configured host and port.
- [x] Add a Mineflayer/raw connection smoke test that verifies TCP accept, handshake, and clean disconnect on a local offline-mode server while Mineflayer play support lags 26.1.2.
- [x] Add a Mineflayer/raw reconnect smoke test that connects, disconnects cleanly, reconnects with the same offline username, and verifies the old connection is fully removed.
- [x] Add a Mineflayer/raw offline-mode login cancellation test that closes the client immediately after login success, during registry sync, and during first chunk delivery, then verifies the next login with the same name is not rejected as duplicate.
- [x] Add a Mineflayer/raw offline-mode wrong-protocol test that connects with an unsupported protocol version and verifies the status response and login disconnect match the decompiled 26.1.2 handshake gate.
- [x] Add a Mineflayer connection-refusal test that attempts login before readiness, during shutdown, and immediately after port close, verifying vanilla-compatible socket errors or disconnect messages.
- [x] Add a Mineflayer offline-mode socket-cleanup test that aborts the TCP socket during handshake, login start, compression negotiation, configuration, and play entry, then verifies no leaked connection slots or pending keepalive tasks.
- [x] Add a Mineflayer offline-mode port-reuse test that starts and stops servers repeatedly on the same randomized port, logs in once per cycle, and verifies no stale listener or TIME_WAIT handling regression blocks the next run.
- [x] Add a Mineflayer parallel-offline-login transport test that starts several generated bots in the same tick window and verifies handshake/login packets are isolated per connection with no cross-bot profile, compression, or keepalive leakage.
- [x] Add a Mineflayer offline-mode login transport-framing test that captures raw packet boundaries around handshake, login success, compression enablement, and configuration entry, then compares official-vs-RustCraft framing and disconnect behavior.
- [x] Add a Mineflayer offline-mode half-open login test that leaves a bot socket idle after TCP connect, after handshake, and after login start, then verifies vanilla-compatible timeout, slot cleanup, and later successful login.
- [x] Implement Netty-equivalent pipeline behavior: frame decode, packet decode, compression, encryption, packet encode, frame encode.
- [x] Implement VarInt and VarLong exactly.
- [x] Implement string, identifier, UUID, optional, collection, enum, bitset, NBT, component, and registry-aware byte buffer codecs.
- [ ] Implement packet size limits and malformed packet disconnect behavior.
- [x] Implement legacy ping/status compatibility if still accepted by 26.1.2 clients.
- [x] Implement compression threshold negotiation and zlib payload handling.
- [x] Implement AES/CFB8 encryption after login key exchange.
- [x] Implement rate limiting and packet flood kicking.
- [x] Implement local memory connection equivalent if needed for integrated tests.
- [x] Implement bundled packet packing/unpacking.
- [x] Implement cookie request/response packets.
- [x] Implement transfer packets.
- [x] Implement custom payload channels and known payload validation.
- [x] Implement keepalive and timeout handling for all relevant protocol states.
- [x] Add a Mineflayer/raw keepalive test that stays connected for multiple heartbeat intervals and verifies no false timeout or duplicate keepalive response handling.
- [x] Add a Mineflayer malformed-client-behavior test that uses Mineflayer packet hooks to send unexpected status, login, configuration, and play packets in offline mode and verifies vanilla-compatible disconnect reasons.
- [x] Add a Mineflayer compression-threshold test that logs in offline mode with disabled, low, and default thresholds, then verifies packet flow still reaches play state and large packets are decoded correctly.
- [x] Implement packet listener dispatch and main-thread handoff rules.
- [x] Implement disconnect messages and close ordering matching vanilla closely enough for clients.

## Migrated From Main Checklist: Protocol State Parity

- [x] Implement handshake state.
- [x] Implement status state.
- [x] Add a Mineflayer/raw 26.1.2 status/ping test that validates MOTD, version, player counts, and latency ping echo shape while Mineflayer play support lags the target protocol.
- [x] Add a Mineflayer/raw status test that compares hidden-player-count and disabled-status behavior against the decompiled 26.1.2 status handshake behavior.
- [x] Add a Mineflayer/raw status-to-login transition test that pings the server, immediately logs in offline mode from the same harness process, and verifies the status socket cleanup cannot corrupt the login connection.
- [x] Implement ping state.
- [x] Implement login state.
- [x] Add a Mineflayer offline-mode login test that reaches login success without Yggdrasil, encryption, or secure-profile requirements.
- [x] Add a Mineflayer offline-mode login test that asserts no session-server HTTP calls are made, no encryption request is sent, and no profile-key packet is required before login success.
- [ ] Add a Mineflayer offline-mode login-order test that captures handshake, login start, compression negotiation, login success, login acknowledgment, configuration packets, finish configuration, and join game ordering against official `server.jar`.
- [x] Add a Mineflayer duplicate-login test that connects two bots with the same offline username and verifies vanilla-compatible replacement or rejection behavior.
- [x] Add a Mineflayer offline-mode returning-login test that joins, disconnects, reconnects with the same username and UUID, and verifies the second login skips first-join initialization that vanilla does not repeat.
- [x] Add a Mineflayer offline-mode username validation test covering valid names, case sensitivity, length limits, illegal characters, duplicate casing, and resulting disconnect messages.
- [x] Add a Mineflayer offline-mode username corpus test that runs a table of vanilla-accepted and vanilla-rejected generated names, asserting UUID derivation, display name preservation, and disconnect component parity.
- [x] Add a Mineflayer offline-mode login timeout test that stalls after handshake, after login start, and during configuration acknowledgment, then verifies vanilla-compatible timeout handling and cleanup.
- [x] Add a Mineflayer offline-mode login reconnect-during-configuration test that drops the TCP connection after login success but before finish-configuration, reconnects, and verifies stale profile/session cleanup.
- [x] Add a Mineflayer offline-mode login disconnect-matrix test that drops the bot at handshake, login start, login success, configuration start, known-packs exchange, and finish-configuration, then verifies server cleanup and log messages.
- [x] Add a Mineflayer offline-mode login retry test that intentionally fails the first attempt with a forced disconnect, immediately retries with the same generated profile, and verifies vanilla-compatible recovery without manual sleeps.
- [x] Add a Mineflayer offline-mode login state-machine invariant test that asserts no play, chat, command, movement, or inventory packet is accepted before the vanilla state transition that permits it.
- [x] Implement configuration state.
- [x] Add a raw 26.1.2 offline-mode configuration/play-entry probe that asserts enabled features, registry identities, non-empty required registries, damage-type tags, finish-configuration, play login, held slot, position, teleport acknowledgement, and player-loaded framing while Mineflayer lacks 26.1.2 protocol support.
- [x] Replace incremental vanilla-client crash chasing with a complete configuration registry closure pass derived from `RegistryDataLoader.SYNCHRONIZED_REGISTRIES`.
- [x] Add a generated synchronized-registry closure report that lists every `RegistryDataLoader.SYNCHRONIZED_REGISTRIES` entry, current RustCraft status, packet source function, codec source file, expected vanilla element count, tag count, and whether the raw probe or Mineflayer can validate it.
- [x] Add a closure gate that fails when a synchronized registry is neither emitted during configuration nor explicitly listed as a milestone-scoped omission with decompiled evidence and a play-entry probe proving the omission is still accepted.
- [ ] Add a decompiled codec audit for each synchronized registry that records required network/direct codec fields, optional defaults, referenced holder/tag fields, and NBT shape before any new registry is marked synced.
- [ ] Add a registry dependency graph test that follows `RegistryDataLoader`, `RegistrySetBuilder`, item component initializers, chunk biome palettes, dimension definitions, command argument types, and first-play packets to identify registries required before vanilla-client join.
- [x] Add a Mineflayer/raw-probe readiness gate that runs after every registry closure change and fails on missing registry packets, missing tag packets, missing client-referenced elements, truncated registry payloads, known-pack ordering drift, or play-state packet decode errors.
- [x] Add an automated decomp audit that extracts every 26.1.2 synchronized registry from `RegistryDataLoader` and fails when RustCraft does not either sync it or document why it is intentionally omitted for the current milestone.
- [x] Add an automated decomp audit that extracts every item `delayedComponent`, `delayedHolderComponent`, `fireResistant`, `jukeboxPlayable`, and registry/tag lookup initializer from `Items.java` and maps each dependency to a configuration registry packet or tag packet.
- [x] Add a vanilla-client configuration completion manifest that records every registry name, element count, element order, tag name, and tag entry index required before `clientbound/minecraft:finish_configuration`.
- [ ] Compare RustCraft configuration registry packets against an official `server.jar` transcript, including packet order, registry IDs, element IDs, element counts, omitted optional fields, tag counts, and tag entry indices.
- [ ] Build an official `server.jar` configuration transcript recorder that performs offline-mode login through configuration, captures enabled features, registry data, update tags, known packs, finish configuration, and first play packets, and stores a normalized fixture.
- [ ] Add a transcript normalizer that removes volatile compression framing, connection IDs, temp paths, timestamps, and random usernames while preserving packet order, registry names, element IDs, NBT field names, tag entry indices, and known-pack tuples.
- [ ] Add a RustCraft-vs-official transcript diff that reports the first registry, tag, or packet-order mismatch with enough packet context to patch the Rust encoder without requiring another manual vanilla-client crash.
- [ ] Add a Mineflayer scenario wrapper that can use the same transcript oracle once prismarine protocol supports 26.1.2, with the raw 26.1.2 probe remaining the fallback until then.
- [x] Extend the raw 26.1.2 probe to parse configuration tag packets and assert required tag registries and tag names, not just registry packet names and counts.
- [x] Extend the raw 26.1.2 probe to parse registry packet element IDs and assert client-referenced elements such as `minecraft:redstone`, `minecraft:cold`, `minecraft:13`, and `minecraft:pattern_item/flower` dependencies are present before finish-configuration.
- [x] Add regression fixtures for every vanilla client crash signature encountered during configuration, keyed by missing registry/tag/element and the decompiled initializer that required it.
- [x] Fix and regress the 26.1.2 `clientbound/minecraft:player_position` relative flag encoding as a fixed 4-byte `INT`, not a VarInt, after vanilla decoded play-state entry but rejected the packet length.
- [x] Fix and regress the 26.1.2 `clientbound/minecraft:level_chunk_with_light` packet ID as `45`, not `48`, after the vanilla client decoded the old ID as `clientbound/minecraft:light_update` and reported 250 trailing bytes.
- [x] Add the initial vanilla play-state world-readiness packets before first chunk delivery: `change_difficulty`, `player_abilities`, `initialize_border`, `set_default_spawn_position`, `set_chunk_cache_center`, `set_chunk_cache_radius`, and `LEVEL_CHUNKS_LOAD_START` `game_event`, after the vanilla client reached `Loading terrain...` but did not enter the world.
- [x] Emit a 3x3 superflat-style spawn chunk batch with non-empty grass sections and sky-light payloads so the vanilla client has initial terrain to render and stand on instead of a void-only chunk.
- [x] Correct level chunk section serialization to the vanilla 26.1.2 field layout after solid sections exposed field-order drift and crashed the vanilla client while reading paletted containers.
- [x] Restore and regress the 26.1.2 `LevelChunkSection` `fluidCount` short after decomp confirmed the section layout is `nonEmptyBlockCount`, `fluidCount`, block states, then biomes, and the vanilla client underflowed at byte 192 while reading the next section.
- [x] Fix and regress 26.1.2 paletted-container chunk serialization to omit VarInt data lengths before fixed long arrays, after the vanilla client saw 240 bytes for a 192-byte all-single-value section payload and underflowed while reading block-state storage.
- [x] Replace the initial all-solid grass spawn section with a vanilla-style superflat bedrock/dirt/grass section using 4-bit local palette storage, so clients spawn above a normal flat surface instead of inside a full solid chunk section.
- [x] Keep the minimal play connection alive after the first in-world render by sending clientbound keepalives and consuming serverbound keepalive replies instead of closing on the status/login socket read timeout.
- [x] Add a raw 26.1.2 play keepalive probe that stays connected past the first heartbeat, replies to `clientbound/minecraft:keep_alive`, and fails on missed heartbeat while Mineflayer lacks 26.1.2 play support.
- [x] Fix and regress the 26.1.2 play-state `clientbound/minecraft:keep_alive` packet ID as `44`, not `113`, after the vanilla client decoded the old ID as `clientbound/minecraft:set_time` and rejected the keepalive payload shape.
- [x] Sync `minecraft:worldgen/biome` during configuration with vanilla-compatible network codec payloads and enough baseline elements for dimension/chunk/client initialization.
- [ ] Correct the configuration biome registry resource key from `minecraft:biome` to vanilla's synchronized `minecraft:worldgen/biome`, verified against the official `server.jar` transcript and the previous vanilla-client crash.
- [ ] Reorder the emitted `minecraft:worldgen/biome` element IDs to match the official `server.jar` transcript so biome holder IDs line up with vanilla.
- [ ] Reorder currently emitted configuration registry data packets to match their relative order in the official `server.jar` transcript: biome, chat type, trim pattern, trim material, wolf, pig, frog, cat, cow, chicken, zombie nautilus, painting, dimension type, damage type, banner pattern, jukebox song, and instrument.
- [x] Expand `minecraft:dimension_type` from only `overworld` to the official `overworld`, `overworld_caves`, `the_end`, and `the_nether` entries with vanilla-compatible network codec payloads.
- [ ] Reorder synced registry element IDs to match the official transcript for cat variants, sound variants, painting variants, damage types, banner patterns, jukebox songs, and instruments.
- [ ] Reorder synced registry element IDs to match the official transcript for chat types, trim materials/patterns, wolf variants, pig variants, frog variants, cow variants, and chicken variants.
- [x] Expand animal sound variant registries from minimal defaults to the official server.jar sets: wolf sound variants `angry`, `big`, `classic`, `cute`, `grumpy`, `puglin`, `sad`; pig sound variants `big`, `classic`, `mini`; chicken/cow/cat sound variants as reported by the transcript oracle.
- [x] Expand `minecraft:painting_variant` from the initial `kebab` placeholder to the 51-entry vanilla data-pack set and assert representative IDs in the Mineflayer raw probe.
- [x] Expand `minecraft:zombie_nautilus_variant` from the initial placeholder to vanilla `temperate` and `warm` entries.
- [ ] Add explicit configuration transcript coverage for the remaining official registries RustCraft currently omits but the vanilla transcript advertises after `banner_pattern`: `minecraft:enchantment`, `minecraft:test_environment`, `minecraft:test_instance`, `minecraft:dialog`, `minecraft:world_clock`, and `minecraft:timeline`.
- [x] Decide whether omitted noncritical registries should remain documented omissions for the join milestone or be synced as empty/full registries before the vanilla-client compatibility milestone is marked complete.
- [x] Extract `Biome` network/direct codec fields from the decompiled server and document the exact NBT payload shape: `Biome.NETWORK_CODEC` includes climate settings, optional positional attributes, and special effects; `Biome.DIRECT_CODEC` additionally includes generation settings and mob spawn settings.
- [x] Add a biome codec fixture that asserts the network payload uses `has_precipitation`, `temperature`, optional `temperature_modifier`, `downfall`, optional `attributes`, and special-effect fields such as `water_color`, optional foliage/dry-foliage/grass overrides, and `grass_color_modifier`.
- [ ] Determine the vanilla biome set required for the current join milestone by tracing dimension type, play login dimension holder IDs, chunk biome palettes, and client biome lookups from the decompiled client/server code, then emit the complete 65-entry vanilla ID set instead of only the minimal subset.
- [x] Emit a baseline `minecraft:worldgen/biome` registry packet with codec-compatible vanilla IDs and field values, starting with the actual biome IDs referenced by the initial chunk payload rather than placeholder data.
- [x] Extend the raw 26.1.2 probe to parse biome registry elements and assert required biome IDs before finish-configuration.
- [x] Extend the raw 26.1.2 probe to verify every emitted biome has the decompiled codec-required fields before finish-configuration.
- [ ] Add a raw 26.1.2 transcript fallback for initial chunk biome palette compatibility while Mineflayer target-protocol chunk observation remains pending. References: `harness/mineflayer/raw_26_1_2_join_probe.mjs`, `harness/mineflayer/raw_26_1_2_chunk_streaming.test.mjs`.
- [x] Sync `minecraft:chat_type` during configuration with vanilla `chat`, `say_command`, `msg_command_incoming`, `msg_command_outgoing`, `team_msg_command_incoming`, `team_msg_command_outgoing`, and `emote_command` entries.
- [ ] Sync `minecraft:trim_pattern` during configuration with all vanilla smithing template patterns and component descriptions.
- [x] Sync `minecraft:trim_material` during configuration with all vanilla trim materials, including `redstone`.
- [x] Sync `minecraft:wolf_variant` during configuration with all vanilla wolf variants.
- [x] Sync `minecraft:wolf_sound_variant` during configuration with a non-empty codec-compatible default.
- [x] Sync `minecraft:pig_variant` during configuration with `temperate`, `warm`, and `cold`.
- [x] Sync `minecraft:pig_sound_variant` during configuration with a non-empty codec-compatible default.
- [x] Sync `minecraft:frog_variant` during configuration with `temperate`, `warm`, and `cold`.
- [x] Sync `minecraft:cat_variant` during configuration with all vanilla cat variants.
- [x] Sync `minecraft:cat_sound_variant` during configuration with a non-empty codec-compatible default.
- [x] Sync `minecraft:cow_variant` during configuration with `temperate`, `warm`, and `cold`.
- [x] Sync `minecraft:cow_sound_variant` during configuration with a non-empty codec-compatible default.
- [x] Sync `minecraft:chicken_variant` during configuration with `temperate`, `warm`, and `cold`.
- [x] Sync `minecraft:chicken_sound_variant` during configuration with a non-empty codec-compatible default.
- [x] Sync `minecraft:zombie_nautilus_variant` during configuration with a non-empty codec-compatible default.
- [x] Sync `minecraft:painting_variant` during configuration with at least the vanilla `kebab` entry and replace the placeholder with the full vanilla set before marking broad client compatibility complete.
- [x] Sync `minecraft:dimension_type` during configuration with an overworld entry compatible with the play login dimension holder ID.
- [x] Sync `minecraft:damage_type` during configuration with every vanilla damage type currently referenced by damage tags.
- [x] Sync `minecraft:banner_pattern` during configuration with all vanilla banner patterns.
- [ ] Sync `minecraft:enchantment` during configuration with all vanilla enchantments or a verified minimal set that satisfies client item component initialization.
- [x] Extract `Enchantment` network/direct codec fields from the decompiled server and document required description, supported items, primary items, exclusive set, weight, max level, cost, anvil cost, slots, and effect component payload shapes.
- [x] Verify whether default item component initialization references any concrete enchantment holder before play-state entry; if not, preserve that result as an automated omission test tied to the raw probe's successful play-entry evidence.
- [x] Decide whether the join milestone should emit zero enchantments, a minimal codec-valid subset, or the full vanilla enchantment registry, and encode that decision in the closure report with a vanilla-client/Mineflayer validation target.
- [x] Add raw probe assertions for the chosen enchantment policy: omitted with documented acceptance, non-empty minimal set with exact IDs, or full vanilla count and element IDs.
- [ ] Add a follow-up Mineflayer enchantment smoke test that joins offline mode, receives an enchanted item or enchanted book, and verifies the bot/client does not hit missing registry, missing tag, tooltip, or component decode failures.
- [x] Sync `minecraft:jukebox_song` during configuration with all vanilla music disc songs.
- [x] Sync `minecraft:instrument` during configuration with all vanilla goat horn instruments.
- [x] Sync `minecraft:test_environment` during configuration or document and verify why the client accepts it omitted for the current milestone.
- [x] Sync `minecraft:test_instance` during configuration or document and verify why the client accepts it omitted for the current milestone.
- [x] Sync `minecraft:dialog` during configuration or document and verify why the client accepts it omitted for the current milestone.
- [x] Sync `minecraft:world_clock` during configuration or document and verify why the client accepts it omitted for the current milestone.
- [x] Sync `minecraft:timeline` during configuration or document and verify why the client accepts it omitted for the current milestone.
- [x] Sync `minecraft:damage_type` tags during configuration, including `minecraft:is_fire`.
- [x] Sync `minecraft:banner_pattern` tags during configuration, including every `minecraft:pattern_item/*` tag used by banner pattern items.
- [ ] Add unit tests for each synced registry that assert exact element counts, client-referenced IDs, and NBT field names expected by the decompiled network/direct codecs.
- [x] Add unit tests for each synced tag registry that assert required tag names and entry indices match the local registry order.
- [ ] Mark the raw 26.1.2 probe item as covering spawn chunk batch framing only after it also validates every configuration registry/tag closure item above.
- [ ] Add a Mineflayer configuration-state test that receives registries, tags, enabled features, known packs, and finish-configuration in vanilla order.
- [ ] Add a Mineflayer configuration regression test that asserts the bot reaches play state only after registry sync, feature flags, tags, and finish-configuration complete.
- [ ] Add a Mineflayer configuration custom-payload test that records unknown payload handling, brand exchange, client information, cookies, and disconnect behavior during offline-mode login.
- [ ] Add a Mineflayer offline-mode configuration replay test that records a vanilla login/configuration transcript and verifies RustCraft reaches the same bot event milestones without hidden sleeps or retry-only success.
- [ ] Implement common state.
- [ ] Implement cookie state.
- [ ] Implement game/play state.
- [ ] Implement all 227 packet classes represented under `net/minecraft/network/protocol`.
- [x] Implement protocol transition from handshake to status.
- [x] Implement protocol transition from handshake to login.
- [ ] Implement login start, encryption request/response, compression, login success, and disconnect.
- [ ] Implement online-mode authentication against Mojang/Yggdrasil services.
- [x] Implement offline-mode UUID derivation.
- [ ] Implement secure profile enforcement.
- [ ] Implement profile key validation and signed chat session setup.
- [ ] Implement configuration registry data sync.
- [x] Implement enabled feature sync.
- [x] Implement known-packs negotiation.
- [x] Implement code of conduct packet behavior.
- [x] Implement resource pack push/pop/status flow.
- [x] Implement server links and dialog packets.
- [ ] Implement game join sequence exactly enough for vanilla client login.
- [ ] Add a Mineflayer join smoke test that waits for spawn, verifies dimension, gamemode, position, health, inventory, and tab-list identity.
- [ ] Add a Mineflayer offline-mode login-to-spawn contract test that asserts the reusable login gate does not pass until the bot has a loaded entity, spawn position, tab-list profile, and first chunk visibility.
- [ ] Add a Mineflayer offline-mode login visibility test that verifies the bot is added to tab list, spawned for nearby bots, and visible to command selectors only after the vanilla play-state boundary.
- [x] Add raw 26.1.2 login visibility boundary fallback coverage that verifies tab-list/player visibility and play login packets are absent during configuration, then the generated offline profile appears in the play tab list with initial chunks ready while Mineflayer lacks target-protocol play support.
- [ ] Add a Mineflayer offline-mode spawn-timeout diagnostic test that forces slow chunk availability and verifies failures report last received chunk, entity ID, dimension, position, and missing readiness milestone.
- [ ] Add a Mineflayer first-spawn test that verifies spawn position, look angles, player abilities, held slot, experience state, and initial time/weather packets.
- [ ] Add a Mineflayer offline-mode post-login readiness test that waits for the first physics tick, then verifies movement, chat, command suggestions, inventory window ID, and chunk visibility are all usable without retry sleeps.
- [x] Add raw 26.1.2 post-login readiness fallback coverage that verifies immediate command-suggestion requests receive a usable `list` suggestion while first movement, chat, inventory, and chunk visibility remain usable without retry sleeps while Mineflayer lacks target-protocol play support.
- [ ] Add a Mineflayer login-to-play timeline test that records bot events from TCP connect through first physics tick and compares ordering, packet gaps, and timeout thresholds against official `server.jar`.
- [x] Add raw 26.1.2 login-to-play timeline fallback coverage that records login, configuration, and first play packet IDs, verifies known-packs/registry/finish-configuration ordering, asserts the vanilla-shaped initial play packet sequence through chunk batch finish, and bounds login-to-first-chunks duration while Mineflayer lacks target-protocol play support.
- [ ] Add a Mineflayer offline-mode first-tick test that verifies the bot can send movement, chat, command, inventory, and block-look packets immediately after play state without race-condition disconnects.
- [x] Add a raw 26.1.2 first-tick passive play packet probe that sends client information, held-slot change, movement, chat, command suggestion, inventory close, block action, swing, use-item-on, use-item, and keepalive immediately after spawn without a race-condition disconnect while Mineflayer lacks 26.1.2 support.
- [ ] Add a Mineflayer offline-mode first-action matrix that sends the first movement, chat, command, inventory click, block dig, and block place immediately after spawn and verifies vanilla-compatible success or correction.
- [x] Add raw 26.1.2 first-action matrix fallback coverage for movement, chat, command suggestion, inventory click, inventory close, block action, player input, swing, use-item-on, and use-item immediately after play entry while Mineflayer lacks target-protocol play support.
- [ ] Add a Mineflayer offline-mode reconnect-at-play-boundary test that disconnects immediately after join game, immediately after first chunk, and immediately after first physics tick, then verifies player cleanup and next login parity.
- [x] Add raw 26.1.2 reconnect-at-play-boundary fallback coverage for join-game, first-chunk, chunk-batch-finished, first-tick-actions, and first-keepalive aborts while Mineflayer lacks target-protocol play support.
- [ ] Add a Mineflayer offline-mode play-readiness race test that repeats login-to-first-action under randomized chunk delays and fails when any action only succeeds after an arbitrary sleep.
- [x] Add raw 26.1.2 play-readiness race fallback coverage that repeats varied immediate first-action combinations through the next keepalive without retry sleeps while Mineflayer lacks target-protocol play support.
- [ ] Implement respawn, dimension change, death, and return-to-game packet flows.
- [ ] Implement chunk batch start/finish and adaptive chunk batching.
- [ ] Implement light update and chunk section serialization.
- [ ] Implement entity spawn, remove, metadata, velocity, teleport, passenger, equipment, attributes, effects, and animation packets.
- [ ] Implement inventory, container, recipe, advancement, statistics, scoreboard, bossbar, title, sound, particle, map, border, command tree, suggestions, and debug packets.

## Migrated From Main Checklist: Source-Derived Granularity Appendix - Protocol Packet Families

- [x] Handshake protocol: implement the 1 packet under `network/protocol/handshake`, including next-state validation and invalid-intent disconnect behavior.
- [x] Status protocol: implement the 2 packets under `network/protocol/status`, including status request/response and ping latency.
- [x] Ping protocol: implement the 2 packets under `network/protocol/ping`, including payload echo and timeout behavior.
- [ ] Login protocol: implement the 9 packets under `network/protocol/login`, including login start, hello/encryption, custom query, compression, success, acknowledgment, cookie request/response, and disconnect.
- [ ] Configuration protocol: implement the 7 packets under `network/protocol/configuration`, including registries, tags, feature flags, known packs, code of conduct, reset chat, and finish configuration.
- [x] Cookie protocol: implement the 2 packets under `network/protocol/cookie`, including cookie request and response correlation.
- [ ] Common protocol: implement the 19 packets under `network/protocol/common`, including keepalive, custom payloads, resource packs, server links, dialogs, ping/pong, tags, transfer, cookies, disconnect, and client information.
- [ ] Game protocol: implement the 182 packets under `network/protocol/game`, grouped into join/respawn, chunks/light, entity lifecycle, entity movement, metadata, inventory/container, recipes, commands/suggestions, chat, scoreboard/team, bossbar, world border, sounds, particles, maps, titles, debug samples, game tests, player abilities, interactions, movement, block/entity actions, resource state, and disconnect.
- [ ] For each packet family, record packet ID, direction, state, field order, codecs, optional fields, registry dependencies, version gates, compression behavior, and disconnect behavior for malformed input.
- [ ] For every clientbound packet, add a golden serialization test generated from vanilla server traffic.
- [ ] For every serverbound packet, add decoder fuzz tests and a replay test that confirms vanilla-compatible side effects.
