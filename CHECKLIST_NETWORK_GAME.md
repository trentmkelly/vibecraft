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
- [x] `ClientboundChangeDifficultyPacket` (0x0B): difficulty byte, difficulty-locked bool
- [x] `ClientboundSetDefaultSpawnPositionPacket` (0x52): BlockPos, angle float
- [x] `ClientboundSetTimePacket` (0x62): game-time long, day-time long, tick-day-time bool
- [x] `ClientboundGameEventPacket` (0x22): event ID byte, param float (covers LEVEL_CHUNKS_LOAD_START, CHANGE_GAME_MODE, WIN_GAME, DEMO_EVENT, ARROW_HIT_PLAYER, RAIN_LEVEL_CHANGE, THUNDER_LEVEL_CHANGE, PUFFER_FISH_STING, GUARDIAN_ELDER_EFFECT, IMMEDIATE_RESPAWN)

## Chunk and Light Packets

- [ ] `ClientboundLevelChunkWithLightPacket` (0x27): chunk X/Z, chunk data (heightmaps, sections, block entities), light data (sky/block light set/empty bitsets, sky/block arrays, trust-edges flag); field order verified against decompiled codec
- [x] `ClientboundForgetLevelChunkPacket` (0x1F): chunk X/Z
- [ ] `ClientboundLightUpdatePacket` (0x28): chunk X/Z, light data; separate from chunk-with-light for mid-game updates
- [x] `ClientboundChunkBatchStartPacket` (0x0D): no fields
- [x] `ClientboundChunkBatchFinishedPacket` (0x0C): batch-size int
- [x] `ServerboundChunkBatchReceivedPacket` (0x09): desired-chunks-per-tick float; adaptive batching feedback
- [x] `ClientboundSetChunkCacheCenterPacket` (0x50): chunk X/Z VarInts
- [x] `ClientboundSetChunkCacheRadiusPacket` (0x51): view-distance VarInt
- [x] `ClientboundSetSimulationDistancePacket` (0x61): simulation-distance VarInt
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
- [x] `ServerboundContainerClosePacket` (0x13): container ID VarInt
- [x] `ServerboundSetCarriedItemPacket` (0x35): slot short
- [x] `ServerboundPickItemFromBlockPacket` (0x24): block pos, include-data bool; `ServerboundPickItemFromEntityPacket` (0x25): entity ID VarInt, include-data bool
- [x] `ServerboundEditBookPacket` (0x18): slot VarInt, pages list capped at 100 entries with 1024-char UTF-8 pages, optional 32-char title
- [x] `ServerboundRenameItemPacket` (0x30): name UTF-8 string capped at 32767 chars
- [x] `ServerboundSelectTradePacket` (0x33): item number VarInt
- [x] `ServerboundSetBeaconPacket` (0x34): primary effect optional MobEffect registry id, secondary effect optional MobEffect registry id
- [ ] `ServerboundSetCreativeModeSlotPacket` (0x2D): slot short, item
- [x] `ServerboundContainerButtonClickPacket` (0x11): container ID VarInt, button ID VarInt

## Recipe / Advancement / Unlock Packets

- [ ] `ClientboundRecipeBookAddPacket` (0x3C): recipe entries list (holder, display, forced-notification), replace flag
- [ ] `ClientboundRecipeBookRemovePacket` (0x3D): recipe IDs list
- [ ] `ClientboundRecipeBookSettingsPacket` (0x3E): recipe book open/filter flags per book type
- [ ] `ClientboundUpdateAdvancementsPacket` (0x74): reset/clear flag, added advancements map (ID → AdvancementHolder), removed advancement IDs, progress map (ID → criterion done-date map)
- [x] `ServerboundRecipeBookChangeSettingsPacket` (0x2E): book type enum VarInt, is-open bool, is-filter-active bool
- [x] `ServerboundRecipeBookSeenRecipePacket` (0x2F): recipe display ID VarInt index

## Commands / Suggestions Packets

- [ ] `ClientboundCommandsPacket` (0x0F): root node, full command tree with argument/literal/redirect nodes and permission flags
- [ ] `ClientboundCommandSuggestionsPacket` (0x10): transaction ID VarInt, range start/length, suggestions list (text, optional tooltip)
- [x] `ServerboundCommandSuggestionPacket` (0x0F): transaction ID VarInt, command UTF-8 string capped at 32500 chars

## Chat Packets

- [ ] `ClientboundPlayerChatPacket` (0x3A): sender UUID, index VarInt, optional signature, message body (plain/formatted), optional unsigned content, filter mask, chat type bound (type + sender name + optional target name)
- [ ] `ClientboundSystemChatPacket` (0x6C): content component, overlay bool
- [ ] `ClientboundDisguisedChatPacket` (0x19): content component, chat type bound
- [ ] `ClientboundDeleteChatPacket` (0x18): message signature bytes
- [ ] `ClientboundPlayerInfoUpdatePacket` (0x3D): action bitmask, entries list (UUID + per-action data: add-player name/properties, initialize-chat session, update-game-mode, update-listed, update-latency, update-display-name, update-hat, update-list-order)
- [ ] `ClientboundPlayerInfoRemovePacket` (0x3C): UUID list
- [x] `ServerboundChatPacket` (0x09): message string max 256, timestamp epoch millis long, salt long, optional 256-byte signature, last-seen update (offset VarInt + fixed 20-bit acknowledgment bitset + checksum byte)
- [x] `ServerboundChatCommandPacket` (0x07): command string max 32767
- [x] `ServerboundChatCommandSignedPacket` (0x08): command string max 32767, timestamp epoch millis long, salt long, argument signatures capped at 8 entries with 16-char names and 256-byte signatures, last-seen update
- [x] `ServerboundChatSessionUpdatePacket` (0x0A): chat session UUID, profile public key data (expires-at epoch millis, public key byte array capped at 512 bytes, signature byte array capped at 4096 bytes)
- [x] `ServerboundChatAckPacket` (0x06): message acknowledgement offset VarInt

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
- [x] `ClientboundSetExperiencePacket` (0x56): experience-progress float, total-experience VarInt, level VarInt
- [x] `ClientboundSetHealthPacket` (0x58): health float, food VarInt, saturation float
- [ ] `ClientboundGameEventPacket` (0x22): (also covers mode-change event 3 = change game mode)
- [ ] `ClientboundPlayerLookAtPacket` (0x39): from-anchor VarInt, target entity-or-block, optional entity-anchor

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
- [ ] `ClientboundPlayerPositionPacket` (0x40): pos X/Y/Z, velocity X/Y/Z, yaw, pitch, relative flags (4-byte INT bitmask — not VarInt), teleport ID VarInt

## Block / Entity Action Packets (Clientbound)

- [ ] `ClientboundBlockUpdatePacket` (0x09): block pos, block state VarInt
- [ ] `ClientboundSectionBlocksUpdatePacket` (0x47): chunk section pos, block states array (packed pos+state longs)
- [ ] `ClientboundBlockEntityDataPacket` (0x07): block pos, type VarInt, NBT tag
- [ ] `ClientboundBlockEventPacket` (0x08): block pos, action byte, param byte, block type VarInt
- [ ] `ClientboundBlockDestructionPacket` (0x06): entity ID, block pos, progress byte (0–9, 10=done)
- [ ] `ClientboundExplodePacket` (0x1D): pos X/Y/Z, radius, affected blocks list (byte offsets), player velocity X/Y/Z, block interaction VarInt, small explosion particle, large explosion particle, sound
- [ ] `ClientboundEntityPositionSyncPacket` (0x20): (26.1.2 name for teleport ack; confirm ID VarInt)
- [x] `ServerboundAcceptTeleportationPacket` (0x00): teleport ID VarInt

## Resource State Packets

- [ ] `ClientboundResourcePackPopPacket`: resource pack UUID
- [ ] `ClientboundResourcePackPushPacket`: UUID, URL, hash, required, optional prompt component
- [x] `ServerboundResourcePackPacket` (0x31): UUID, action enum VarInt (`SUCCESSFULLY_LOADED`, `DECLINED`, `FAILED_DOWNLOAD`, `ACCEPTED`, `DOWNLOADED`, `INVALID_URL`, `FAILED_RELOAD`, `DISCARDED`)

## Debug / Misc Packets

- [ ] `ClientboundDebugSamplePacket` (0x17): sample array longs, sample type VarInt
- [ ] `ClientboundCustomPayloadPacket` (0x15) — `minecraft:brand` and other channels
- [ ] `ClientboundStartConfigurationPacket` (0x69): triggers switch from play back to configuration state
- [x] `ServerboundConfigurationAcknowledgedPacket` (0x10): empty terminal payload; triggers play→configuration ack
- [x] `ClientboundPingPacket` (0x3D): ID int
- [x] `ServerboundPongPacket` (0x2D): ID int
- [ ] `ClientboundDisconnectPacket` (0x1B): reason component
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

- [ ] Implement TCP listener on configured host and port.
- [ ] Add a Mineflayer/raw connection smoke test that verifies TCP accept, handshake, and clean disconnect on a local offline-mode server while Mineflayer play support lags 26.1.2.
- [ ] Add a Mineflayer/raw reconnect smoke test that connects, disconnects cleanly, reconnects with the same offline username, and verifies the old connection is fully removed.
- [ ] Add a Mineflayer/raw offline-mode login cancellation test that closes the client immediately after login success, during registry sync, and during first chunk delivery, then verifies the next login with the same name is not rejected as duplicate.
- [ ] Add a Mineflayer/raw offline-mode wrong-protocol test that connects with an unsupported protocol version and verifies the status response and login disconnect match the decompiled 26.1.2 handshake gate.
- [ ] Add a Mineflayer connection-refusal test that attempts login before readiness, during shutdown, and immediately after port close, verifying vanilla-compatible socket errors or disconnect messages.
- [ ] Add a Mineflayer offline-mode socket-cleanup test that aborts the TCP socket during handshake, login start, compression negotiation, configuration, and play entry, then verifies no leaked connection slots or pending keepalive tasks.
- [ ] Add a Mineflayer offline-mode port-reuse test that starts and stops servers repeatedly on the same randomized port, logs in once per cycle, and verifies no stale listener or TIME_WAIT handling regression blocks the next run.
- [ ] Add a Mineflayer parallel-offline-login transport test that starts several generated bots in the same tick window and verifies handshake/login packets are isolated per connection with no cross-bot profile, compression, or keepalive leakage.
- [ ] Add a Mineflayer offline-mode login transport-framing test that captures raw packet boundaries around handshake, login success, compression enablement, and configuration entry, then compares official-vs-RustCraft framing and disconnect behavior.
- [ ] Add a Mineflayer offline-mode half-open login test that leaves a bot socket idle after TCP connect, after handshake, and after login start, then verifies vanilla-compatible timeout, slot cleanup, and later successful login.
- [ ] Implement Netty-equivalent pipeline behavior: frame decode, packet decode, compression, encryption, packet encode, frame encode.
- [ ] Implement VarInt and VarLong exactly.
- [ ] Implement string, identifier, UUID, optional, collection, enum, bitset, NBT, component, and registry-aware byte buffer codecs.
- [ ] Implement packet size limits and malformed packet disconnect behavior.
- [ ] Implement legacy ping/status compatibility if still accepted by 26.1.2 clients.
- [ ] Implement compression threshold negotiation and zlib payload handling.
- [ ] Implement AES/CFB8 encryption after login key exchange.
- [ ] Implement rate limiting and packet flood kicking.
- [ ] Implement local memory connection equivalent if needed for integrated tests.
- [ ] Implement bundled packet packing/unpacking.
- [ ] Implement cookie request/response packets.
- [ ] Implement transfer packets.
- [ ] Implement custom payload channels and known payload validation.
- [ ] Implement keepalive and timeout handling for all relevant protocol states.
- [ ] Add a Mineflayer/raw keepalive test that stays connected for multiple heartbeat intervals and verifies no false timeout or duplicate keepalive response handling.
- [ ] Add a Mineflayer malformed-client-behavior test that uses Mineflayer packet hooks to send unexpected status, login, configuration, and play packets in offline mode and verifies vanilla-compatible disconnect reasons.
- [ ] Add a Mineflayer compression-threshold test that logs in offline mode with disabled, low, and default thresholds, then verifies packet flow still reaches play state and large packets are decoded correctly.
- [ ] Implement packet listener dispatch and main-thread handoff rules.
- [ ] Implement disconnect messages and close ordering matching vanilla closely enough for clients.

## Migrated From Main Checklist: Protocol State Parity

- [ ] Implement handshake state.
- [ ] Implement status state.
- [ ] Add a Mineflayer/raw 26.1.2 status/ping test that validates MOTD, version, player counts, and latency ping echo shape while Mineflayer play support lags the target protocol.
- [ ] Add a Mineflayer/raw status test that compares hidden-player-count and disabled-status behavior against the decompiled 26.1.2 status handshake behavior.
- [ ] Add a Mineflayer/raw status-to-login transition test that pings the server, immediately logs in offline mode from the same harness process, and verifies the status socket cleanup cannot corrupt the login connection.
- [ ] Implement ping state.
- [ ] Implement login state.
- [ ] Add a Mineflayer offline-mode login test that reaches login success without Yggdrasil, encryption, or secure-profile requirements.
- [ ] Add a Mineflayer offline-mode login test that asserts no session-server HTTP calls are made, no encryption request is sent, and no profile-key packet is required before login success.
- [ ] Add a Mineflayer offline-mode login-order test that captures handshake, login start, compression negotiation, login success, login acknowledgment, configuration packets, finish configuration, and join game ordering against official `server.jar`.
- [ ] Add a Mineflayer duplicate-login test that connects two bots with the same offline username and verifies vanilla-compatible replacement or rejection behavior.
- [ ] Add a Mineflayer offline-mode returning-login test that joins, disconnects, reconnects with the same username and UUID, and verifies the second login skips first-join initialization that vanilla does not repeat.
- [ ] Add a Mineflayer offline-mode username validation test covering valid names, case sensitivity, length limits, illegal characters, duplicate casing, and resulting disconnect messages.
- [ ] Add a Mineflayer offline-mode username corpus test that runs a table of vanilla-accepted and vanilla-rejected generated names, asserting UUID derivation, display name preservation, and disconnect component parity.
- [ ] Add a Mineflayer offline-mode login timeout test that stalls after handshake, after login start, and during configuration acknowledgment, then verifies vanilla-compatible timeout handling and cleanup.
- [ ] Add a Mineflayer offline-mode login reconnect-during-configuration test that drops the TCP connection after login success but before finish-configuration, reconnects, and verifies stale profile/session cleanup.
- [ ] Add a Mineflayer offline-mode login disconnect-matrix test that drops the bot at handshake, login start, login success, configuration start, known-packs exchange, and finish-configuration, then verifies server cleanup and log messages.
- [ ] Add a Mineflayer offline-mode login retry test that intentionally fails the first attempt with a forced disconnect, immediately retries with the same generated profile, and verifies vanilla-compatible recovery without manual sleeps.
- [ ] Add a Mineflayer offline-mode login state-machine invariant test that asserts no play, chat, command, movement, or inventory packet is accepted before the vanilla state transition that permits it.
- [ ] Implement configuration state.
- [ ] Add a raw 26.1.2 offline-mode configuration/play-entry probe that asserts enabled features, registry identities, non-empty required registries, damage-type tags, finish-configuration, play login, held slot, position, teleport acknowledgement, and player-loaded framing while Mineflayer lacks 26.1.2 protocol support.
- [ ] Replace incremental vanilla-client crash chasing with a complete configuration registry closure pass derived from `RegistryDataLoader.SYNCHRONIZED_REGISTRIES`.
- [ ] Add a generated synchronized-registry closure report that lists every `RegistryDataLoader.SYNCHRONIZED_REGISTRIES` entry, current RustCraft status, packet source function, codec source file, expected vanilla element count, tag count, and whether the raw probe or Mineflayer can validate it.
- [ ] Add a closure gate that fails when a synchronized registry is neither emitted during configuration nor explicitly listed as a milestone-scoped omission with decompiled evidence and a play-entry probe proving the omission is still accepted.
- [ ] Add a decompiled codec audit for each synchronized registry that records required network/direct codec fields, optional defaults, referenced holder/tag fields, and NBT shape before any new registry is marked synced.
- [ ] Add a registry dependency graph test that follows `RegistryDataLoader`, `RegistrySetBuilder`, item component initializers, chunk biome palettes, dimension definitions, command argument types, and first-play packets to identify registries required before vanilla-client join.
- [ ] Add a Mineflayer/raw-probe readiness gate that runs after every registry closure change and fails on missing registry packets, missing tag packets, missing client-referenced elements, truncated registry payloads, known-pack ordering drift, or play-state packet decode errors.
- [ ] Add an automated decomp audit that extracts every 26.1.2 synchronized registry from `RegistryDataLoader` and fails when RustCraft does not either sync it or document why it is intentionally omitted for the current milestone.
- [ ] Add an automated decomp audit that extracts every item `delayedComponent`, `delayedHolderComponent`, `fireResistant`, `jukeboxPlayable`, and registry/tag lookup initializer from `Items.java` and maps each dependency to a configuration registry packet or tag packet.
- [ ] Add a vanilla-client configuration completion manifest that records every registry name, element count, element order, tag name, and tag entry index required before `clientbound/minecraft:finish_configuration`.
- [ ] Compare RustCraft configuration registry packets against an official `server.jar` transcript, including packet order, registry IDs, element IDs, element counts, omitted optional fields, tag counts, and tag entry indices.
- [ ] Build an official `server.jar` configuration transcript recorder that performs offline-mode login through configuration, captures enabled features, registry data, update tags, known packs, finish configuration, and first play packets, and stores a normalized fixture.
- [ ] Add a transcript normalizer that removes volatile compression framing, connection IDs, temp paths, timestamps, and random usernames while preserving packet order, registry names, element IDs, NBT field names, tag entry indices, and known-pack tuples.
- [ ] Add a RustCraft-vs-official transcript diff that reports the first registry, tag, or packet-order mismatch with enough packet context to patch the Rust encoder without requiring another manual vanilla-client crash.
- [ ] Add a Mineflayer scenario wrapper that can use the same transcript oracle once prismarine protocol supports 26.1.2, with the raw 26.1.2 probe remaining the fallback until then.
- [ ] Extend the raw 26.1.2 probe to parse configuration tag packets and assert required tag registries and tag names, not just registry packet names and counts.
- [ ] Extend the raw 26.1.2 probe to parse registry packet element IDs and assert client-referenced elements such as `minecraft:redstone`, `minecraft:cold`, `minecraft:13`, and `minecraft:pattern_item/flower` dependencies are present before finish-configuration.
- [ ] Add regression fixtures for every vanilla client crash signature encountered during configuration, keyed by missing registry/tag/element and the decompiled initializer that required it.
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
- [ ] Sync `minecraft:worldgen/biome` during configuration with vanilla-compatible network codec payloads and enough baseline elements for dimension/chunk/client initialization.
- [ ] Correct the configuration biome registry resource key from `minecraft:biome` to vanilla's synchronized `minecraft:worldgen/biome`, verified against the official `server.jar` transcript and the previous vanilla-client crash.
- [ ] Reorder the emitted `minecraft:worldgen/biome` element IDs to match the official `server.jar` transcript so biome holder IDs line up with vanilla.
- [ ] Reorder currently emitted configuration registry data packets to match their relative order in the official `server.jar` transcript: biome, chat type, trim pattern, trim material, wolf, pig, frog, cat, cow, chicken, zombie nautilus, painting, dimension type, damage type, banner pattern, jukebox song, and instrument.
- [ ] Expand `minecraft:dimension_type` from only `overworld` to the official `overworld`, `overworld_caves`, `the_end`, and `the_nether` entries with vanilla-compatible network codec payloads.
- [ ] Reorder synced registry element IDs to match the official transcript for cat variants, sound variants, painting variants, damage types, banner patterns, jukebox songs, and instruments.
- [ ] Reorder synced registry element IDs to match the official transcript for chat types, trim materials/patterns, wolf variants, pig variants, frog variants, cow variants, and chicken variants.
- [ ] Expand animal sound variant registries from minimal defaults to the official server.jar sets: wolf sound variants `angry`, `big`, `classic`, `cute`, `grumpy`, `puglin`, `sad`; pig sound variants `big`, `classic`, `mini`; chicken/cow/cat sound variants as reported by the transcript oracle.
- [ ] Expand `minecraft:painting_variant` from the initial `kebab` placeholder to the 51-entry vanilla data-pack set and assert representative IDs in the Mineflayer raw probe.
- [ ] Expand `minecraft:zombie_nautilus_variant` from the initial placeholder to vanilla `temperate` and `warm` entries.
- [ ] Add explicit configuration transcript coverage for the remaining official registries RustCraft currently omits but the vanilla transcript advertises after `banner_pattern`: `minecraft:enchantment`, `minecraft:test_environment`, `minecraft:test_instance`, `minecraft:dialog`, `minecraft:world_clock`, and `minecraft:timeline`.
- [ ] Decide whether omitted noncritical registries should remain documented omissions for the join milestone or be synced as empty/full registries before the vanilla-client compatibility milestone is marked complete.
- [ ] Extract `Biome` network/direct codec fields from the decompiled server and document the exact NBT payload shape: `Biome.NETWORK_CODEC` includes climate settings, optional positional attributes, and special effects; `Biome.DIRECT_CODEC` additionally includes generation settings and mob spawn settings.
- [ ] Add a biome codec fixture that asserts the network payload uses `has_precipitation`, `temperature`, optional `temperature_modifier`, `downfall`, optional `attributes`, and special-effect fields such as `water_color`, optional foliage/dry-foliage/grass overrides, and `grass_color_modifier`.
- [ ] Determine the vanilla biome set required for the current join milestone by tracing dimension type, play login dimension holder IDs, chunk biome palettes, and client biome lookups from the decompiled client/server code, then emit the complete 65-entry vanilla ID set instead of only the minimal subset.
- [ ] Emit a baseline `minecraft:worldgen/biome` registry packet with codec-compatible vanilla IDs and field values, starting with the actual biome IDs referenced by the initial chunk payload rather than placeholder data.
- [ ] Extend the raw 26.1.2 probe to parse biome registry elements and assert required biome IDs before finish-configuration.
- [ ] Extend the raw 26.1.2 probe to verify every emitted biome has the decompiled codec-required fields before finish-configuration.
- [ ] Add a raw 26.1.2 transcript fallback for initial chunk biome palette compatibility while Mineflayer target-protocol chunk observation remains pending. References: `harness/mineflayer/raw_26_1_2_join_probe.mjs`, `harness/mineflayer/raw_26_1_2_chunk_streaming.test.mjs`.
- [ ] Sync `minecraft:chat_type` during configuration with vanilla `chat`, `say_command`, `msg_command_incoming`, `msg_command_outgoing`, `team_msg_command_incoming`, `team_msg_command_outgoing`, and `emote_command` entries.
- [ ] Sync `minecraft:trim_pattern` during configuration with all vanilla smithing template patterns and component descriptions.
- [ ] Sync `minecraft:trim_material` during configuration with all vanilla trim materials, including `redstone`.
- [ ] Sync `minecraft:wolf_variant` during configuration with all vanilla wolf variants.
- [ ] Sync `minecraft:wolf_sound_variant` during configuration with a non-empty codec-compatible default.
- [ ] Sync `minecraft:pig_variant` during configuration with `temperate`, `warm`, and `cold`.
- [ ] Sync `minecraft:pig_sound_variant` during configuration with a non-empty codec-compatible default.
- [ ] Sync `minecraft:frog_variant` during configuration with `temperate`, `warm`, and `cold`.
- [ ] Sync `minecraft:cat_variant` during configuration with all vanilla cat variants.
- [ ] Sync `minecraft:cat_sound_variant` during configuration with a non-empty codec-compatible default.
- [ ] Sync `minecraft:cow_variant` during configuration with `temperate`, `warm`, and `cold`.
- [ ] Sync `minecraft:cow_sound_variant` during configuration with a non-empty codec-compatible default.
- [ ] Sync `minecraft:chicken_variant` during configuration with `temperate`, `warm`, and `cold`.
- [ ] Sync `minecraft:chicken_sound_variant` during configuration with a non-empty codec-compatible default.
- [ ] Sync `minecraft:zombie_nautilus_variant` during configuration with a non-empty codec-compatible default.
- [ ] Sync `minecraft:painting_variant` during configuration with at least the vanilla `kebab` entry and replace the placeholder with the full vanilla set before marking broad client compatibility complete.
- [ ] Sync `minecraft:dimension_type` during configuration with an overworld entry compatible with the play login dimension holder ID.
- [ ] Sync `minecraft:damage_type` during configuration with every vanilla damage type currently referenced by damage tags.
- [ ] Sync `minecraft:banner_pattern` during configuration with all vanilla banner patterns.
- [ ] Sync `minecraft:enchantment` during configuration with all vanilla enchantments or a verified minimal set that satisfies client item component initialization.
- [ ] Extract `Enchantment` network/direct codec fields from the decompiled server and document required description, supported items, primary items, exclusive set, weight, max level, cost, anvil cost, slots, and effect component payload shapes.
- [ ] Verify whether default item component initialization references any concrete enchantment holder before play-state entry; if not, preserve that result as an automated omission test tied to the raw probe's successful play-entry evidence.
- [ ] Decide whether the join milestone should emit zero enchantments, a minimal codec-valid subset, or the full vanilla enchantment registry, and encode that decision in the closure report with a vanilla-client/Mineflayer validation target.
- [ ] Add raw probe assertions for the chosen enchantment policy: omitted with documented acceptance, non-empty minimal set with exact IDs, or full vanilla count and element IDs.
- [ ] Add a follow-up Mineflayer enchantment smoke test that joins offline mode, receives an enchanted item or enchanted book, and verifies the bot/client does not hit missing registry, missing tag, tooltip, or component decode failures.
- [ ] Sync `minecraft:jukebox_song` during configuration with all vanilla music disc songs.
- [ ] Sync `minecraft:instrument` during configuration with all vanilla goat horn instruments.
- [ ] Sync `minecraft:test_environment` during configuration or document and verify why the client accepts it omitted for the current milestone.
- [ ] Sync `minecraft:test_instance` during configuration or document and verify why the client accepts it omitted for the current milestone.
- [ ] Sync `minecraft:dialog` during configuration or document and verify why the client accepts it omitted for the current milestone.
- [ ] Sync `minecraft:world_clock` during configuration or document and verify why the client accepts it omitted for the current milestone.
- [ ] Sync `minecraft:timeline` during configuration or document and verify why the client accepts it omitted for the current milestone.
- [ ] Sync `minecraft:damage_type` tags during configuration, including `minecraft:is_fire`.
- [ ] Sync `minecraft:banner_pattern` tags during configuration, including every `minecraft:pattern_item/*` tag used by banner pattern items.
- [ ] Add unit tests for each synced registry that assert exact element counts, client-referenced IDs, and NBT field names expected by the decompiled network/direct codecs.
- [ ] Add unit tests for each synced tag registry that assert required tag names and entry indices match the local registry order.
- [ ] Mark the raw 26.1.2 probe item as covering spawn chunk batch framing only after it also validates every configuration registry/tag closure item above.
- [ ] Add a Mineflayer configuration-state test that receives registries, tags, enabled features, known packs, and finish-configuration in vanilla order.
- [ ] Add a Mineflayer configuration regression test that asserts the bot reaches play state only after registry sync, feature flags, tags, and finish-configuration complete.
- [ ] Add a Mineflayer configuration custom-payload test that records unknown payload handling, brand exchange, client information, cookies, and disconnect behavior during offline-mode login.
- [ ] Add a Mineflayer offline-mode configuration replay test that records a vanilla login/configuration transcript and verifies RustCraft reaches the same bot event milestones without hidden sleeps or retry-only success.
- [ ] Implement common state.
- [ ] Implement cookie state.
- [ ] Implement game/play state.
- [ ] Implement all 227 packet classes represented under `net/minecraft/network/protocol`.
- [ ] Implement protocol transition from handshake to status.
- [ ] Implement protocol transition from handshake to login.
- [ ] Implement login start, encryption request/response, compression, login success, and disconnect.
- [ ] Implement online-mode authentication against Mojang/Yggdrasil services.
- [ ] Implement offline-mode UUID derivation.
- [ ] Implement secure profile enforcement.
- [ ] Implement profile key validation and signed chat session setup.
- [ ] Implement configuration registry data sync.
- [ ] Implement enabled feature sync.
- [ ] Implement known-packs negotiation.
- [ ] Implement code of conduct packet behavior.
- [ ] Implement resource pack push/pop/status flow.
- [ ] Implement server links and dialog packets.
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
- [ ] Add a raw 26.1.2 first-tick passive play packet probe that sends client information, held-slot change, movement, chat, command suggestion, inventory close, block action, swing, use-item-on, use-item, and keepalive immediately after spawn without a race-condition disconnect while Mineflayer lacks 26.1.2 support.
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

- [ ] Handshake protocol: implement the 1 packet under `network/protocol/handshake`, including next-state validation and invalid-intent disconnect behavior.
- [ ] Status protocol: implement the 2 packets under `network/protocol/status`, including status request/response and ping latency.
- [ ] Ping protocol: implement the 2 packets under `network/protocol/ping`, including payload echo and timeout behavior.
- [ ] Login protocol: implement the 9 packets under `network/protocol/login`, including login start, hello/encryption, custom query, compression, success, acknowledgment, cookie request/response, and disconnect.
- [ ] Configuration protocol: implement the 7 packets under `network/protocol/configuration`, including registries, tags, feature flags, known packs, code of conduct, reset chat, and finish configuration.
- [ ] Cookie protocol: implement the 2 packets under `network/protocol/cookie`, including cookie request and response correlation.
- [ ] Common protocol: implement the 19 packets under `network/protocol/common`, including keepalive, custom payloads, resource packs, server links, dialogs, ping/pong, tags, transfer, cookies, disconnect, and client information.
- [ ] Game protocol: implement the 182 packets under `network/protocol/game`, grouped into join/respawn, chunks/light, entity lifecycle, entity movement, metadata, inventory/container, recipes, commands/suggestions, chat, scoreboard/team, bossbar, world border, sounds, particles, maps, titles, debug samples, game tests, player abilities, interactions, movement, block/entity actions, resource state, and disconnect.
- [ ] For each packet family, record packet ID, direction, state, field order, codecs, optional fields, registry dependencies, version gates, compression behavior, and disconnect behavior for malformed input.
- [ ] For every clientbound packet, add a golden serialization test generated from vanilla server traffic.
- [ ] For every serverbound packet, add decoder fuzz tests and a replay test that confirms vanilla-compatible side effects.
