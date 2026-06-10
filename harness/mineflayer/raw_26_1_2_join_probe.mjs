import net from 'node:net'
import crypto from 'node:crypto'
import { deflateSync, inflateSync } from 'node:zlib'

const host = process.env.VIBECRAFT_HOST ?? '127.0.0.1'
const port = Number(process.env.VIBECRAFT_PORT ?? 25565)
const username = process.env.VIBECRAFT_USERNAME ?? 'VibeCraftProbe'
const protocolVersion = Number(process.env.VIBECRAFT_PROTOCOL_VERSION ?? 775)
const recordOnly = process.env.VIBECRAFT_RAW_PROBE_MODE === 'record'
const summaryOnly = process.env.VIBECRAFT_RAW_PROBE_OUTPUT === 'summary'
const expectLoginDisconnect = process.env.VIBECRAFT_EXPECT_LOGIN_DISCONNECT === '1'
const abortAfter = process.env.VIBECRAFT_RAW_PROBE_ABORT_AFTER ?? ''
const keepAliveProbeMs = Number(process.env.VIBECRAFT_RAW_PROBE_KEEPALIVE_MS ?? 0)
const postActionProbeMs = Number(process.env.VIBECRAFT_RAW_PROBE_POST_ACTION_MS ?? 0)
const firstTickActionRequest = process.env.VIBECRAFT_RAW_PROBE_FIRST_TICK_ACTIONS ?? ''
const firstTickActions = new Set(firstTickActionRequest === '1'
  ? ['client_information', 'held_slot', 'movement', 'chat', 'command_suggestion', 'inventory_click', 'inventory_close', 'block_action', 'player_input', 'swing', 'use_item_on', 'use_item']
  : firstTickActionRequest.split(',').map(action => action.trim()).filter(Boolean))
const expectedJoinPosition = parsePositionEnv(process.env.VIBECRAFT_EXPECT_JOIN_POSITION, null)
const expectedDefaultSpawn = parseBlockPosEnv(process.env.VIBECRAFT_EXPECT_DEFAULT_SPAWN, null)
const movementPosition = parsePositionEnv(process.env.VIBECRAFT_RAW_PROBE_MOVEMENT_POSITION, { x: 0.5, y: 80, z: 0.5, yaw: 0, pitch: 0 })
const extraMovementPositions = parsePositionArrayEnv(process.env.VIBECRAFT_RAW_PROBE_EXTRA_MOVEMENTS)
const expectedHeldSlot = Number(process.env.VIBECRAFT_EXPECT_HELD_SLOT ?? 0)
const carriedItemSlot = Number(process.env.VIBECRAFT_RAW_PROBE_HELD_SLOT ?? 4)
const expectedHealth = Number(process.env.VIBECRAFT_EXPECT_HEALTH ?? 20)
const expectedFoodLevel = Number(process.env.VIBECRAFT_EXPECT_FOOD_LEVEL ?? 20)
const expectedFoodSaturation = Number(process.env.VIBECRAFT_EXPECT_FOOD_SATURATION ?? 5)
const expectedXpProgress = Number(process.env.VIBECRAFT_EXPECT_XP_PROGRESS ?? 0)
const expectedXpLevel = Number(process.env.VIBECRAFT_EXPECT_XP_LEVEL ?? 0)
const expectedXpTotal = Number(process.env.VIBECRAFT_EXPECT_XP_TOTAL ?? 0)
const expectedGameMode = Number(process.env.VIBECRAFT_EXPECT_GAME_MODE ?? 0)
const expectedPreviousGameMode = Number(process.env.VIBECRAFT_EXPECT_PREVIOUS_GAME_MODE ?? 255)
const expectedAbilityFlags = Number(process.env.VIBECRAFT_EXPECT_ABILITY_FLAGS ?? 0)
const expectedCommandSuggestion = process.env.VIBECRAFT_EXPECT_COMMAND_SUGGESTION ?? ''
const expectedWorldSeed = process.env.VIBECRAFT_EXPECT_WORLD_SEED == null
  ? null
  : BigInt(process.env.VIBECRAFT_EXPECT_WORLD_SEED)
const expectedIsFlat = process.env.VIBECRAFT_EXPECT_IS_FLAT == null
  ? null
  : parseBoolEnv(process.env.VIBECRAFT_EXPECT_IS_FLAT, true)
const serverboundAcceptTeleportationPacketId = 0
const clientboundCommandSuggestionsPacketId = 15
const serverboundChatPacketId = 9
const serverboundChunkBatchReceivedPacketId = 11
const serverboundClientInformationPacketId = 14
const serverboundCommandSuggestionPacketId = 15
const serverboundContainerClickPacketId = 18
const serverboundContainerClosePacketId = 19
const serverboundKeepAlivePacketId = 28
const serverboundMovePlayerPosRotPacketId = 31
const serverboundPlayerActionPacketId = 41
const serverboundPlayerInputPacketId = 43
const serverboundSelectKnownPacksPacketId = 7
const serverboundSetCarriedItemPacketId = 53
const serverboundSwingPacketId = 63
const serverboundUseItemOnPacketId = 66
const serverboundUseItemPacketId = 67
const serverboundPlayerLoadedPacketId = 44
const clientboundKeepAlivePacketId = 44

function writeVarInt (value) {
  let remaining = value >>> 0
  const bytes = []
  do {
    let byte = remaining & 0x7f
    remaining >>>= 7
    if (remaining !== 0) byte |= 0x80
    bytes.push(byte)
  } while (remaining !== 0)
  return Buffer.from(bytes)
}

function readVarInt (buffer, offset = 0) {
  let value = 0
  let shift = 0
  let cursor = offset
  while (cursor < buffer.length) {
    const byte = buffer[cursor++]
    value |= (byte & 0x7f) << shift
    if ((byte & 0x80) === 0) return { value, offset: cursor }
    shift += 7
    if (shift > 35) throw new Error('varint too large')
  }
  return null
}

function writeString (value) {
  const data = Buffer.from(value, 'utf8')
  return Buffer.concat([writeVarInt(data.length), data])
}

function writeShort (value) {
  const payload = Buffer.alloc(2)
  payload.writeInt16BE(value)
  return payload
}

function readString (buffer, offset = 0) {
  const length = readVarInt(buffer, offset)
  if (!length) throw new Error('missing string length')
  const end = length.offset + length.value
  if (buffer.length < end) throw new Error('truncated string')
  return { value: buffer.subarray(length.offset, end).toString('utf8'), offset: end }
}

function offlineUuid (name) {
  const hash = crypto.createHash('md5').update(`OfflinePlayer:${name}`, 'utf8').digest()
  hash[6] = (hash[6] & 0x0f) | 0x30
  hash[8] = (hash[8] & 0x3f) | 0x80
  const hex = hash.toString('hex')
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`
}

function readUuid (buffer, offset = 0) {
  const hex = buffer.subarray(offset, offset + 16).toString('hex')
  return {
    value: `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`,
    offset: offset + 16
  }
}

function readBlockPos (buffer, offset = 0) {
  const packed = buffer.readBigInt64BE(offset)
  const x = Number(BigInt.asIntN(26, packed >> 38n))
  const y = Number(BigInt.asIntN(12, packed))
  const z = Number(BigInt.asIntN(26, packed >> 12n))
  return { x, y, z, offset: offset + 8 }
}

function nearlyEqual (actual, expected, epsilon = 0.000001) {
  return Math.abs(actual - expected) <= epsilon
}

function parseBoolEnv (value, fallback) {
  if (value == null || value === '') return fallback
  return value === '1' || value === 'true'
}

function frame (packetId, ...parts) {
  const payload = Buffer.concat([writeVarInt(packetId), ...parts])
  return Buffer.concat([writeVarInt(payload.length), payload])
}

function randomUuidBytes () {
  return Buffer.from(crypto.randomUUID().replaceAll('-', ''), 'hex')
}

function writeKnownPack (pack) {
  return Buffer.concat([writeString(pack.namespace), writeString(pack.id), writeString(pack.version)])
}

class PacketReader {
  constructor (socket) {
    this.socket = socket
    this.buffer = Buffer.alloc(0)
    this.waiters = []
    this.compressionThreshold = null
    socket.on('data', chunk => {
      this.buffer = Buffer.concat([this.buffer, chunk])
      this.pump()
    })
    socket.on('error', error => this.rejectAll(error))
    socket.on('close', () => this.rejectAll(new Error('socket closed')))
  }

  nextPacket () {
    return new Promise((resolve, reject) => {
      this.waiters.push({ resolve, reject })
      this.pump()
    })
  }

  pump () {
    while (this.waiters.length > 0) {
      const length = readVarInt(this.buffer)
      if (!length) return
      const end = length.offset + length.value
      if (this.buffer.length < end) return
      const payload = this.buffer.subarray(length.offset, end)
      this.buffer = this.buffer.subarray(end)
      const decoded = this.decodePayload(payload)
      const packetId = readVarInt(decoded.payload)
      if (!packetId) throw new Error('packet missing id')
      this.waiters.shift().resolve({
        id: packetId.value,
        body: decoded.payload.subarray(packetId.offset),
        length: length.value,
        compressed: decoded.compressed
      })
    }
  }

  decodePayload (payload) {
    if (this.compressionThreshold == null) return { payload, compressed: false }
    const dataLength = readVarInt(payload)
    if (!dataLength) throw new Error('compressed frame missing data length')
    const body = payload.subarray(dataLength.offset)
    if (dataLength.value === 0) return { payload: body, compressed: false }
    const inflated = inflateSync(body)
    if (inflated.length !== dataLength.value) {
      throw new Error(`decompressed packet length mismatch ${inflated.length} != ${dataLength.value}`)
    }
    return { payload: inflated, compressed: true }
  }

  setCompression (threshold) {
    this.compressionThreshold = threshold
  }

  rejectAll (error) {
    for (const waiter of this.waiters.splice(0)) waiter.reject(error)
  }
}

function nextPacketWithin (reader, timeoutMs) {
  return Promise.race([
    reader.nextPacket().then(packet => ({ packet })),
    new Promise(resolve => setTimeout(() => resolve({ timeout: true }), timeoutMs))
  ])
}

function expectPacket (packet, id, state) {
  if (packet.id !== id) {
    throw new Error(`expected ${state} packet ${id}, got ${packet.id}`)
  }
}

function encodeClientPacket (reader, packetId, ...parts) {
  const payload = Buffer.concat([writeVarInt(packetId), ...parts])
  const threshold = reader.compressionThreshold
  if (threshold == null) return Buffer.concat([writeVarInt(payload.length), payload])
  if (payload.length < threshold) {
    const compressedPayload = Buffer.concat([writeVarInt(0), payload])
    return Buffer.concat([writeVarInt(compressedPayload.length), compressedPayload])
  }
  const compressed = deflateSync(payload)
  const compressedPayload = Buffer.concat([writeVarInt(payload.length), compressed])
  return Buffer.concat([writeVarInt(compressedPayload.length), compressedPayload])
}

const expectedRegistries = [
  'minecraft:worldgen/biome',
  'minecraft:damage_type',
  'minecraft:dimension_type',
  'minecraft:chat_type',
  'minecraft:trim_pattern',
  'minecraft:trim_material',
  'minecraft:banner_pattern',
  'minecraft:instrument',
  'minecraft:jukebox_song',
  'minecraft:cat_sound_variant',
  'minecraft:cat_variant',
  'minecraft:chicken_sound_variant',
  'minecraft:chicken_variant',
  'minecraft:cow_sound_variant',
  'minecraft:cow_variant',
  'minecraft:frog_variant',
  'minecraft:painting_variant',
  'minecraft:pig_sound_variant',
  'minecraft:pig_variant',
  'minecraft:wolf_sound_variant',
  'minecraft:wolf_variant',
  'minecraft:zombie_nautilus_variant',
  'minecraft:world_clock',
  'minecraft:timeline'
]

const minimumRegistryElements = new Map([
  ['minecraft:banner_pattern', 43],
  ['minecraft:worldgen/biome', 65],
  ['minecraft:cat_sound_variant', 2],
  ['minecraft:cat_variant', 11],
  ['minecraft:chat_type', 7],
  ['minecraft:chicken_sound_variant', 2],
  ['minecraft:chicken_variant', 3],
  ['minecraft:cow_sound_variant', 2],
  ['minecraft:cow_variant', 3],
  ['minecraft:damage_type', 50],
  ['minecraft:dimension_type', 4],
  ['minecraft:frog_variant', 3],
  ['minecraft:instrument', 8],
  ['minecraft:jukebox_song', 21],
  ['minecraft:painting_variant', 51],
  ['minecraft:pig_sound_variant', 3],
  ['minecraft:pig_variant', 3],
  ['minecraft:timeline', 4],
  ['minecraft:trim_material', 11],
  ['minecraft:trim_pattern', 18],
  ['minecraft:world_clock', 2],
  ['minecraft:wolf_sound_variant', 7],
  ['minecraft:wolf_variant', 9],
  ['minecraft:zombie_nautilus_variant', 2]
])

const requiredRegistryElements = new Map([
  ['minecraft:worldgen/biome', [
    'minecraft:plains',
    'minecraft:the_void',
    'minecraft:end_barrens'
  ]],
  ['minecraft:chat_type', [
    'minecraft:chat',
    'minecraft:emote_command',
    'minecraft:msg_command_incoming',
    'minecraft:msg_command_outgoing',
    'minecraft:say_command',
    'minecraft:team_msg_command_incoming',
    'minecraft:team_msg_command_outgoing'
  ]],
  ['minecraft:banner_pattern', [
    'minecraft:flower',
    'minecraft:bricks',
    'minecraft:curly_border'
  ]],
  ['minecraft:cat_sound_variant', ['minecraft:classic', 'minecraft:royal']],
  ['minecraft:chicken_variant', [
    'minecraft:cold',
    'minecraft:temperate',
    'minecraft:warm'
  ]],
  ['minecraft:chicken_sound_variant', ['minecraft:classic', 'minecraft:picky']],
  ['minecraft:cow_sound_variant', ['minecraft:classic', 'minecraft:moody']],
  ['minecraft:damage_type', ['minecraft:spear']],
  ['minecraft:dimension_type', [
    'minecraft:overworld',
    'minecraft:overworld_caves',
    'minecraft:the_end',
    'minecraft:the_nether'
  ]],
  ['minecraft:instrument', ['minecraft:ponder_goat_horn']],
  ['minecraft:jukebox_song', [
    'minecraft:11',
    'minecraft:13',
    'minecraft:5',
    'minecraft:creator',
    'minecraft:creator_music_box',
    'minecraft:lava_chicken',
    'minecraft:precipice',
    'minecraft:tears'
  ]],
  ['minecraft:painting_variant', ['minecraft:alban', 'minecraft:kebab', 'minecraft:wither']],
  ['minecraft:pig_sound_variant', ['minecraft:big', 'minecraft:classic', 'minecraft:mini']],
  ['minecraft:trim_material', [
    'minecraft:amethyst',
    'minecraft:copper',
    'minecraft:diamond',
    'minecraft:emerald',
    'minecraft:gold',
    'minecraft:iron',
    'minecraft:lapis',
    'minecraft:netherite',
    'minecraft:quartz',
    'minecraft:redstone',
    'minecraft:resin'
  ]],
  ['minecraft:wolf_sound_variant', ['minecraft:angry', 'minecraft:big', 'minecraft:classic', 'minecraft:cute', 'minecraft:grumpy', 'minecraft:puglin', 'minecraft:sad']],
  ['minecraft:zombie_nautilus_variant', ['minecraft:temperate', 'minecraft:warm']]
])

const requiredTags = new Map([
  ['minecraft:banner_pattern', [
    'minecraft:pattern_item/bordure_indented',
    'minecraft:pattern_item/creeper',
    'minecraft:pattern_item/field_masoned',
    'minecraft:pattern_item/flower',
    'minecraft:pattern_item/flow',
    'minecraft:pattern_item/globe',
    'minecraft:pattern_item/guster',
    'minecraft:pattern_item/mojang',
    'minecraft:pattern_item/piglin',
    'minecraft:pattern_item/skull'
  ]],
  ['minecraft:damage_type', [
    'minecraft:bypasses_shield',
    'minecraft:is_explosion',
    'minecraft:is_fire'
  ]]
])

const requiredBiomeFieldPaths = [
  'has_precipitation',
  'temperature',
  'downfall',
  'effects',
  'effects.water_color'
]

function decodeRegistryPacket (packet) {
  const registry = readString(packet.body)
  const count = readVarInt(packet.body, registry.offset)
  if (!count) throw new Error(`missing element count for registry ${registry.value}`)
  let offset = count.offset
  const elements = []
  const elementDataFields = {}
  for (let i = 0; i < count.value; i++) {
    const element = readString(packet.body, offset)
    elements.push(element.value)
    offset = element.offset
    const hasData = packet.body[offset++]
    if (hasData === 1) {
      const nbt = readNetworkNbt(packet.body, offset)
      offset = nbt.offset
      if (registry.value === 'minecraft:worldgen/biome') {
        elementDataFields[element.value] = collectNbtFieldPaths(nbt.value)
      }
    } else if (hasData !== 0) {
      throw new Error(`invalid registry data marker ${hasData} for ${registry.value}/${element.value}`)
    }
  }
  if (offset !== packet.body.length) {
    throw new Error(`registry ${registry.value} had ${packet.body.length - offset} trailing bytes`)
  }
  return {
    id: packet.id,
    length: packet.length,
    registry: registry.value,
    elements: count.value,
    elementIds: elements,
    ...(Object.keys(elementDataFields).length > 0 ? { elementDataFields } : {})
  }
}

function decodeTagsPacket (packet) {
  const registryCount = readVarInt(packet.body)
  if (!registryCount) throw new Error('missing tag registry count')
  let offset = registryCount.offset
  const registries = []
  for (let i = 0; i < registryCount.value; i++) {
    const registry = readString(packet.body, offset)
    offset = registry.offset
    const tagCount = readVarInt(packet.body, offset)
    if (!tagCount) throw new Error(`missing tag count for ${registry.value}`)
    offset = tagCount.offset
    const tags = []
    for (let j = 0; j < tagCount.value; j++) {
      const tag = readString(packet.body, offset)
      offset = tag.offset
      const entryCount = readVarInt(packet.body, offset)
      if (!entryCount) throw new Error(`missing tag entry count for ${registry.value}/${tag.value}`)
      offset = entryCount.offset
      const entries = []
      for (let k = 0; k < entryCount.value; k++) {
        const entry = readVarInt(packet.body, offset)
        if (!entry) throw new Error(`missing tag entry ${k} for ${registry.value}/${tag.value}`)
        entries.push(entry.value)
        offset = entry.offset
      }
      tags.push({ tag: tag.value, entries })
    }
    registries.push({ registry: registry.value, tags })
  }
  if (offset !== packet.body.length) {
    throw new Error(`tags packet had ${packet.body.length - offset} trailing bytes`)
  }
  return { id: packet.id, length: packet.length, registries }
}

function decodeKnownPacksPacket (packet) {
  const count = readVarInt(packet.body)
  if (!count) throw new Error('missing known pack count')
  let offset = count.offset
  const packs = []
  for (let i = 0; i < count.value; i++) {
    const namespace = readString(packet.body, offset)
    const id = readString(packet.body, namespace.offset)
    const version = readString(packet.body, id.offset)
    offset = version.offset
    packs.push({ namespace: namespace.value, id: id.value, version: version.value })
  }
  if (offset !== packet.body.length) {
    throw new Error(`known packs packet had ${packet.body.length - offset} trailing bytes`)
  }
  return { id: packet.id, length: packet.length, packs }
}

function decodeEnabledFeaturesPacket (packet) {
  const count = readVarInt(packet.body)
  if (!count) throw new Error('missing enabled feature count')
  let offset = count.offset
  const features = []
  for (let i = 0; i < count.value; i++) {
    const feature = readString(packet.body, offset)
    offset = feature.offset
    features.push(feature.value)
  }
  if (offset !== packet.body.length) {
    throw new Error(`enabled features packet had ${packet.body.length - offset} trailing bytes`)
  }
  return { id: packet.id, length: packet.length, features }
}

function skipNetworkNbt (buffer, offset) {
  const type = buffer[offset++]
  if (type !== 10) throw new Error(`expected compound NBT tag, got ${type}`)
  return skipNbtPayload(buffer, offset, type)
}

function readNetworkNbt (buffer, offset) {
  const type = buffer[offset++]
  if (type !== 10) throw new Error(`expected compound NBT tag, got ${type}`)
  return readNbtPayload(buffer, offset, type)
}

function readNbtPayload (buffer, offset, type) {
  switch (type) {
    case 0:
      return { value: null, offset }
    case 1:
      return { value: buffer.readInt8(offset), offset: offset + 1 }
    case 2:
      return { value: buffer.readInt16BE(offset), offset: offset + 2 }
    case 3:
      return { value: buffer.readInt32BE(offset), offset: offset + 4 }
    case 4:
      return { value: Number(buffer.readBigInt64BE(offset)), offset: offset + 8 }
    case 5:
      return { value: buffer.readFloatBE(offset), offset: offset + 4 }
    case 6:
      return { value: buffer.readDoubleBE(offset), offset: offset + 8 }
    case 7: {
      const length = buffer.readInt32BE(offset)
      return { value: [...buffer.subarray(offset + 4, offset + 4 + length)], offset: offset + 4 + length }
    }
    case 8: {
      const length = buffer.readUInt16BE(offset)
      return { value: buffer.toString('utf8', offset + 2, offset + 2 + length), offset: offset + 2 + length }
    }
    case 9: {
      const childType = buffer[offset]
      const length = buffer.readInt32BE(offset + 1)
      let cursor = offset + 5
      const values = []
      for (let i = 0; i < length; i++) {
        const child = readNbtPayload(buffer, cursor, childType)
        values.push(child.value)
        cursor = child.offset
      }
      return { value: values, offset: cursor }
    }
    case 10: {
      let cursor = offset
      const value = {}
      while (true) {
        const childType = buffer[cursor++]
        if (childType === 0) return { value, offset: cursor }
        const nameLength = buffer.readUInt16BE(cursor)
        const name = buffer.toString('utf8', cursor + 2, cursor + 2 + nameLength)
        cursor += 2 + nameLength
        const child = readNbtPayload(buffer, cursor, childType)
        value[name] = child.value
        cursor = child.offset
      }
    }
    case 11: {
      const length = buffer.readInt32BE(offset)
      const values = []
      let cursor = offset + 4
      for (let i = 0; i < length; i++) {
        values.push(buffer.readInt32BE(cursor))
        cursor += 4
      }
      return { value: values, offset: cursor }
    }
    case 12: {
      const length = buffer.readInt32BE(offset)
      const values = []
      let cursor = offset + 4
      for (let i = 0; i < length; i++) {
        values.push(Number(buffer.readBigInt64BE(cursor)))
        cursor += 8
      }
      return { value: values, offset: cursor }
    }
    default:
      throw new Error(`unsupported NBT tag ${type}`)
  }
}

function collectNbtFieldPaths (value, prefix = '') {
  if (!value || Array.isArray(value) || typeof value !== 'object') return []
  const paths = []
  for (const [key, child] of Object.entries(value)) {
    const path = prefix ? `${prefix}.${key}` : key
    paths.push(path)
    paths.push(...collectNbtFieldPaths(child, path))
  }
  return paths
}

function skipNbtPayload (buffer, offset, type) {
  switch (type) {
    case 0:
      return offset
    case 1:
      return offset + 1
    case 2:
      return offset + 2
    case 3:
    case 5:
      return offset + 4
    case 4:
    case 6:
      return offset + 8
    case 7: {
      const length = buffer.readInt32BE(offset)
      return offset + 4 + length
    }
    case 8: {
      const length = buffer.readUInt16BE(offset)
      return offset + 2 + length
    }
    case 9: {
      const childType = buffer[offset]
      const length = buffer.readInt32BE(offset + 1)
      let cursor = offset + 5
      for (let i = 0; i < length; i++) cursor = skipNbtPayload(buffer, cursor, childType)
      return cursor
    }
    case 10: {
      let cursor = offset
      while (true) {
        const childType = buffer[cursor++]
        if (childType === 0) return cursor
        const nameLength = buffer.readUInt16BE(cursor)
        cursor += 2 + nameLength
        cursor = skipNbtPayload(buffer, cursor, childType)
      }
    }
    case 11: {
      const length = buffer.readInt32BE(offset)
      return offset + 4 + length * 4
    }
    case 12: {
      const length = buffer.readInt32BE(offset)
      return offset + 4 + length * 8
    }
    default:
      throw new Error(`unsupported NBT tag ${type}`)
  }
}

async function main () {
  const socket = net.createConnection({ host, port })
  await new Promise((resolve, reject) => {
    socket.once('connect', resolve)
    socket.once('error', reject)
  })
  const reader = new PacketReader(socket)
  const serverAddress = Buffer.concat([
    writeVarInt(protocolVersion),
    writeString(host),
    Buffer.from([(port >> 8) & 0xff, port & 0xff]),
    writeVarInt(2)
  ])
  socket.write(frame(0, serverAddress))
  socket.write(frame(0, writeString(username), randomUuidBytes()))

  let login = await reader.nextPacket()
  if (expectLoginDisconnect) {
    expectPacket(login, 0, 'login_disconnect')
    socket.end()
    const reason = readString(login.body, 0)
    console.log(JSON.stringify({ ok: true, disconnected: true, host, port, login: login.id, length: login.length, reason: reason.value }, null, 2))
    return
  }
  let compressionThreshold = null
  if (login.id === 3) {
    const threshold = readVarInt(login.body, 0)
    if (!threshold) throw new Error('missing login compression threshold')
    compressionThreshold = threshold.value
    reader.setCompression(compressionThreshold)
    login = await reader.nextPacket()
  }
  expectPacket(login, 2, 'login_finished')
  if (abortAfter === 'login_success') return abortSocket(socket, 'login_success', { login: login.id, compressionThreshold })
  socket.write(encodeClientPacket(reader, 3))

  const config = []
  while (true) {
    const packet = await reader.nextPacket()
    if (packet.id === 7) {
      config.push(decodeRegistryPacket(packet))
    } else if (packet.id === 12) {
      config.push(decodeEnabledFeaturesPacket(packet))
    } else if (packet.id === 13) {
      config.push(decodeTagsPacket(packet))
    } else if (packet.id === 14) {
      const knownPacks = decodeKnownPacksPacket(packet)
      config.push(knownPacks)
      if (abortAfter === 'known_packs') return abortSocket(socket, 'known_packs', { login: login.id, config })
      socket.write(encodeClientPacket(
        reader,
        serverboundSelectKnownPacksPacketId,
        writeVarInt(knownPacks.packs.length),
        ...knownPacks.packs.map(writeKnownPack)
      ))
    } else {
      config.push({ id: packet.id, length: packet.length })
    }
    if (abortAfter === 'registry_sync' && packet.id === 7) return abortSocket(socket, 'registry_sync', { login: login.id, config })
    if (abortAfter === 'finish_configuration' && packet.id === 3) return abortSocket(socket, 'finish_configuration', { login: login.id, config })
    if (packet.id === 3) break
  }
  const registryPackets = config.filter(packet => packet.id === 7)
  if (!recordOnly) {
    const configIds = config.map(packet => packet.id)
    for (const id of [12, 7, 13, 14, 3]) {
      if (!configIds.includes(id)) throw new Error(`missing configuration packet ${id}`)
    }
    const knownPacksPacket = config.find(packet => packet.id === 14)
    if (!knownPacksPacket) throw new Error('missing select_known_packs packet')
    if (!knownPacksPacket.packs.some(pack => pack.namespace === 'minecraft' && pack.id === 'core' && pack.version === '26.1.2')) {
      throw new Error('missing minecraft:core:26.1.2 known pack')
    }
    const registryNames = new Set(registryPackets.map(packet => packet.registry))
    if (registryNames.has('minecraft:enchantment')) {
      throw new Error('enchantment registry must remain omitted until enchanted-item smoke coverage exists')
    }
    for (const registry of expectedRegistries) {
      if (!registryNames.has(registry)) throw new Error(`missing registry packet ${registry}`)
    }
    for (const packet of registryPackets) {
      const minimum = minimumRegistryElements.get(packet.registry) ?? 1
      if (packet.elements < minimum) {
        throw new Error(`registry ${packet.registry} had ${packet.elements} elements, expected at least ${minimum}`)
      }
    }
    for (const [registry, elements] of requiredRegistryElements) {
      const packet = registryPackets.find(packet => packet.registry === registry)
      if (!packet) throw new Error(`missing required-element registry ${registry}`)
      const packetElements = new Set(packet.elementIds)
      for (const element of elements) {
        if (!packetElements.has(element)) throw new Error(`missing registry element ${registry}/${element}`)
      }
    }
    const biomePacket = registryPackets.find(packet => packet.registry === 'minecraft:worldgen/biome')
    if (!biomePacket) throw new Error('missing biome registry for field validation')
    for (const element of biomePacket.elementIds) {
      const fields = new Set(biomePacket.elementDataFields?.[element] ?? [])
      for (const field of requiredBiomeFieldPaths) {
        if (!fields.has(field)) throw new Error(`biome ${element} missing network codec field ${field}`)
      }
    }
    const tagsPacket = config.find(packet => packet.id === 13)
    if (!tagsPacket) throw new Error('missing update_tags packet')
    const tagRegistries = new Map(tagsPacket.registries.map(registry => [registry.registry, registry.tags]))
    for (const [registry, tags] of requiredTags) {
      const registryTags = tagRegistries.get(registry)
      if (!registryTags) throw new Error(`missing tag registry ${registry}`)
      const tagNames = new Set(registryTags.map(tag => tag.tag))
      for (const tag of tags) {
        if (!tagNames.has(tag)) throw new Error(`missing tag ${registry}/${tag}`)
        const packetTag = registryTags.find(packetTag => packetTag.tag === tag)
        if (packetTag.entries.length === 0) throw new Error(`tag ${registry}/${tag} had no entries`)
      }
    }
  }
  socket.write(encodeClientPacket(reader, 3))

  const play = []
  const playPackets = []
  const joinState = {}
  let keepAliveReplies = 0
  const expectedPlayPacketPrefixIds = [49, 70, 10, 64, 105, 103, 104, 76, 18, 96, 113, 94, 95, 72, 43, 97, 38, 38, 38, 38, 12]
  // The server-side view distance can send up to a 33x33 initial chunk window
  // before the chunk_batch_finished marker. Keep this above that ceiling so the
  // probe does not report a false missing packet when view-distance is high.
  const maxPlayPacketsBeforeJoinReady = 2048
  for (let i = 0; i < maxPlayPacketsBeforeJoinReady;) {
    const packet = await reader.nextPacket()
    if (packet.id === clientboundKeepAlivePacketId) {
      if (packet.body.length !== 8) {
        throw new Error(`expected 8-byte keep_alive payload, got ${packet.body.length}`)
      }
      keepAliveReplies += 1
      play.push({ id: packet.id, length: packet.length })
      socket.write(encodeClientPacket(reader, serverboundKeepAlivePacketId, packet.body))
      continue
    }
    playPackets.push(packet)
    play.push({ id: packet.id, length: packet.length })
    if (i < expectedPlayPacketPrefixIds.length && packet.id !== expectedPlayPacketPrefixIds[i] && !recordOnly) {
      throw new Error(`expected play packet ${expectedPlayPacketPrefixIds[i]} at index ${i}, got ${packet.id}`)
    }
    if (abortAfter === 'join_game' && packet.id === 49) return abortSocket(socket, 'join_game', { login: login.id, config, play })
    if (abortAfter === 'first_chunk' && packet.id === 45) return abortSocket(socket, 'first_chunk', { login: login.id, config, play })
    if (abortAfter === 'chunk_batch_finished' && packet.id === 11) return abortSocket(socket, 'chunk_batch_finished', { login: login.id, config, play })
    i += 1
    if (packet.id === 11 && i > expectedPlayPacketPrefixIds.length) break
  }
  if (!recordOnly) {
    for (const id of [...expectedPlayPacketPrefixIds, 11]) {
      if (!play.some(packet => packet.id === id)) throw new Error(`missing play packet ${id}`)
    }
    if (!play.some(packet => packet.id === 45)) throw new Error('missing initial level_chunk_with_light packet')
    const packetById = new Map()
    for (const packet of playPackets) {
      if (!packetById.has(packet.id)) packetById.set(packet.id, [])
      packetById.get(packet.id).push(packet)
    }
    const loginPacket = packetById.get(49)?.[0]
    if (!loginPacket || loginPacket.body.length < 69) {
      throw new Error(`expected play login body after holder-id encoding, got ${loginPacket?.body.length}`)
    }
    joinState.entityId = loginPacket.body.readInt32BE(0)
    joinState.dimension = 'minecraft:overworld'
    const loginSpawn = readLoginSpawnInfo(loginPacket.body)
    if (loginSpawn.gameMode !== expectedGameMode || loginSpawn.previousGameMode !== expectedPreviousGameMode) {
      throw new Error(`expected login gameMode=${expectedGameMode} previous=${expectedPreviousGameMode}, got ${loginSpawn.gameMode}/${loginSpawn.previousGameMode}`)
    }
    if ((expectedWorldSeed != null && loginSpawn.seed !== expectedWorldSeed) || (expectedIsFlat != null && loginSpawn.isFlat !== expectedIsFlat) || loginSpawn.seaLevel !== 63) {
      throw new Error(`unexpected login spawn info seed=${loginSpawn.seed} isFlat=${loginSpawn.isFlat} seaLevel=${loginSpawn.seaLevel}`)
    }
    joinState.loginSpawnInfo = {
      seed: loginSpawn.seed.toString(),
      gameMode: loginSpawn.gameMode,
      previousGameMode: loginSpawn.previousGameMode,
      isDebug: loginSpawn.isDebug,
      isFlat: loginSpawn.isFlat,
      seaLevel: loginSpawn.seaLevel
    }
    joinState.loginDistances = {
      viewDistance: loginSpawn.chunkRadius,
      simulationDistance: loginSpawn.simulationDistance
    }
    const playerInfoPacket = packetById.get(70)?.[0]
    if (!playerInfoPacket) throw new Error('missing player_info_update packet')
    let playerInfoOffset = 0
    const actionMask = playerInfoPacket.body[playerInfoOffset++]
    const entryCount = readVarInt(playerInfoPacket.body, playerInfoOffset)
    if (actionMask !== 0xff || !entryCount || entryCount.value !== 1) {
      throw new Error(`expected initializing player_info_update action mask and one entry, got mask=${actionMask} entries=${entryCount?.value}`)
    }
    playerInfoOffset = entryCount.offset
    const profileId = readUuid(playerInfoPacket.body, playerInfoOffset); playerInfoOffset = profileId.offset
    const profileName = readString(playerInfoPacket.body, playerInfoOffset); playerInfoOffset = profileName.offset
    const propertiesCount = readVarInt(playerInfoPacket.body, playerInfoOffset); playerInfoOffset = propertiesCount.offset
    const chatSessionPresent = playerInfoPacket.body[playerInfoOffset++]
    const gameMode = readVarInt(playerInfoPacket.body, playerInfoOffset); playerInfoOffset = gameMode.offset
    const listed = playerInfoPacket.body[playerInfoOffset++]
    const latency = readVarInt(playerInfoPacket.body, playerInfoOffset); playerInfoOffset = latency.offset
    const displayNamePresent = playerInfoPacket.body[playerInfoOffset++]
    const listOrder = readVarInt(playerInfoPacket.body, playerInfoOffset); playerInfoOffset = listOrder.offset
    const showHat = playerInfoPacket.body[playerInfoOffset++]
    if (profileId.value !== offlineUuid(username) || profileName.value !== username || propertiesCount.value !== 0 || chatSessionPresent !== 0 || gameMode.value !== expectedGameMode || listed !== 1 || latency.value !== 0 || displayNamePresent !== 0 || listOrder.value !== 0 || showHat !== 1 || playerInfoOffset !== playerInfoPacket.body.length) {
      throw new Error('unexpected player_info_update identity or tab-list payload')
    }
    joinState.profile = { name: profileName.value, uuid: profileId.value }
    const abilitiesPacket = packetById.get(64)?.[0]
    if (!abilitiesPacket || abilitiesPacket.body.length !== 9) {
      throw new Error(`expected 9-byte player_abilities body, got ${abilitiesPacket?.body.length}`)
    }
    if (abilitiesPacket.body[0] !== expectedAbilityFlags || !nearlyEqual(abilitiesPacket.body.readFloatBE(1), 0.05) || !nearlyEqual(abilitiesPacket.body.readFloatBE(5), 0.1)) {
      throw new Error('unexpected first-spawn player abilities payload')
    }
    const heldSlotPacket = packetById.get(105)?.[0]
    const heldSlot = heldSlotPacket && readVarInt(heldSlotPacket.body)
    if (!heldSlot || heldSlot.value !== expectedHeldSlot || heldSlot.offset !== heldSlotPacket.body.length) {
      throw new Error(`expected selected hotbar slot ${expectedHeldSlot}`)
    }
    const experiencePacket = packetById.get(103)?.[0]
    if (!experiencePacket || experiencePacket.body.length < 6 || !nearlyEqual(experiencePacket.body.readFloatBE(0), expectedXpProgress)) {
      throw new Error(`expected set_experience progress=${expectedXpProgress}, got ${experiencePacket?.body.toString('hex')}`)
    }
    const experienceLevel = readVarInt(experiencePacket.body, 4)
    const totalExperience = experienceLevel && readVarInt(experiencePacket.body, experienceLevel.offset)
    if (!experienceLevel || !totalExperience || experienceLevel.value !== expectedXpLevel || totalExperience.value !== expectedXpTotal || totalExperience.offset !== experiencePacket.body.length) {
      throw new Error(`expected experience level=${expectedXpLevel} total=${expectedXpTotal}`)
    }
    const healthPacket = packetById.get(104)?.[0]
    const food = healthPacket && readVarInt(healthPacket.body, 4)
    if (!healthPacket || healthPacket.body.length < 9 || !nearlyEqual(healthPacket.body.readFloatBE(0), expectedHealth) || !food || food.value !== expectedFoodLevel || !nearlyEqual(healthPacket.body.readFloatBE(food.offset), expectedFoodSaturation)) {
      throw new Error(`expected health=${expectedHealth} food=${expectedFoodLevel} saturation=${expectedFoodSaturation}, got ${healthPacket?.body.toString('hex')}`)
    }
    const inventoryPacket = packetById.get(18)?.[0]
    if (!inventoryPacket || inventoryPacket.body[0] !== 0) throw new Error('expected player inventory container content for container 0')
    const inventoryState = readVarInt(inventoryPacket.body, 1)
    const itemCount = inventoryState && readVarInt(inventoryPacket.body, inventoryState.offset)
    if (!inventoryState || inventoryState.value !== 0 || !itemCount || itemCount.value !== 46) {
      throw new Error('expected empty 46-slot player inventory baseline')
    }
    let inventoryOffset = itemCount.offset
    for (let slot = 0; slot < 46; slot++) {
      const item = readVarInt(inventoryPacket.body, inventoryOffset)
      if (!item || item.value !== 0) throw new Error(`expected empty item stack at inventory slot ${slot}`)
      inventoryOffset = item.offset
    }
    const carried = readVarInt(inventoryPacket.body, inventoryOffset)
    if (!carried || carried.value !== 0 || carried.offset !== inventoryPacket.body.length) {
      throw new Error('expected empty carried inventory item')
    }
    const cursorPacket = packetById.get(96)?.[0]
    const cursorItem = cursorPacket && readVarInt(cursorPacket.body)
    if (!cursorItem || cursorItem.value !== 0 || cursorItem.offset !== cursorPacket.body.length) {
      throw new Error('expected empty cursor item')
    }
    const timePacket = packetById.get(113)?.[0]
    const clockCount = timePacket && readVarInt(timePacket.body, 8)
    if (!timePacket || timePacket.body.length < 9 || timePacket.body.readBigInt64BE(0) < 0n || !clockCount) {
      throw new Error(`expected set_time with non-negative gameTime, got ${timePacket?.body.toString('hex')}`)
    }
    if (clockCount.value === 0 && clockCount.offset !== timePacket.body.length) {
      throw new Error(`expected empty set_time clock map to end at byte ${clockCount.offset}, got ${timePacket.body.toString('hex')}`)
    }
    if (clockCount.value > 0 && clockCount.offset >= timePacket.body.length) {
      throw new Error(`expected full set_time clock sync payload after ${clockCount.value} clocks, got ${timePacket.body.toString('hex')}`)
    }
    const positionPacket = packetById.get(72)?.[0]
    if (!positionPacket || positionPacket.body.length < 61) {
      throw new Error(`expected player_position body with fixed-int relatives, got ${positionPacket?.body.length}`)
    }
    let offset = 0
    const teleportId = readVarInt(positionPacket.body, offset)
    if (!teleportId || teleportId.value !== 0) throw new Error('expected teleport id 0')
    offset = teleportId.offset
    const x = positionPacket.body.readDoubleBE(offset); offset += 8
    const y = positionPacket.body.readDoubleBE(offset); offset += 8
    const z = positionPacket.body.readDoubleBE(offset); offset += 8
    offset += 24
    const yaw = positionPacket.body.readFloatBE(offset); offset += 4
    const pitch = positionPacket.body.readFloatBE(offset); offset += 4
    const relatives = positionPacket.body.length - offset >= 4
      ? positionPacket.body.readInt32BE(offset)
      : positionPacket.body.readUInt8(offset)
    if (expectedJoinPosition != null && (
      !nearlyEqual(x, expectedJoinPosition.x) ||
      !nearlyEqual(y, expectedJoinPosition.y) ||
      !nearlyEqual(z, expectedJoinPosition.z) ||
      !nearlyEqual(yaw, expectedJoinPosition.yaw) ||
      !nearlyEqual(pitch, expectedJoinPosition.pitch) ||
      relatives !== 0
    )) {
      throw new Error('unexpected first-spawn position/look payload')
    }
    joinState.position = { x, y, z, yaw, pitch }
    const spawnPacket = packetById.get(97)?.[0]
    if (!spawnPacket) throw new Error('missing default spawn position packet')
    const spawnDimension = readString(spawnPacket.body, 0)
    const spawnPos = readBlockPos(spawnPacket.body, spawnDimension.offset)
    if (spawnDimension.value !== 'minecraft:overworld') {
      throw new Error(`unexpected default spawn dimension ${spawnDimension.value}`)
    }
    if (expectedDefaultSpawn != null && (spawnPos.x !== expectedDefaultSpawn.x || spawnPos.y !== expectedDefaultSpawn.y || spawnPos.z !== expectedDefaultSpawn.z)) {
      throw new Error(`unexpected default spawn ${spawnDimension.value} ${spawnPos.x} ${spawnPos.y} ${spawnPos.z}`)
    }
    joinState.defaultSpawn = {
      dimension: spawnDimension.value,
      x: spawnPos.x,
      y: spawnPos.y,
      z: spawnPos.z
    }
    const gameEvents = packetById.get(38) ?? []
    const gameEventPairs = gameEvents.map(packet => [packet.body[0], packet.body.readFloatBE(1)])
    for (const expected of [[2, 0], [7, 0], [8, 0], [13, 0]]) {
      if (!gameEventPairs.some(([event, param]) => event === expected[0] && param === expected[1])) {
        throw new Error(`missing first-spawn game_event ${expected[0]}=${expected[1]}`)
      }
    }
    const chunkCacheRadiusPacket = packetById.get(95)?.[0]
    const chunkCacheRadius = chunkCacheRadiusPacket && readVarInt(chunkCacheRadiusPacket.body)
    if (!chunkCacheRadius || chunkCacheRadius.offset !== chunkCacheRadiusPacket.body.length) {
      throw new Error('missing chunk cache radius payload')
    }
    const chunkCacheCenterPacket = packetById.get(94)?.[0]
    const chunkCacheCenterX = chunkCacheCenterPacket && readVarInt(chunkCacheCenterPacket.body)
    const chunkCacheCenterZ = chunkCacheCenterX && readVarInt(chunkCacheCenterPacket.body, chunkCacheCenterX.offset)
    if (!chunkCacheCenterX || !chunkCacheCenterZ || chunkCacheCenterZ.offset !== chunkCacheCenterPacket.body.length) {
      throw new Error('missing chunk cache center payload')
    }
    const chunkBatchFinishedPacket = packetById.get(11)?.[0]
    const chunkBatchSize = chunkBatchFinishedPacket && readVarInt(chunkBatchFinishedPacket.body)
    if (!chunkBatchSize || chunkBatchSize.offset !== chunkBatchFinishedPacket.body.length) {
      throw new Error('missing chunk batch finished payload')
    }
    joinState.chunkStreaming = {
      cacheCenter: {
        x: chunkCacheCenterX.value,
        z: chunkCacheCenterZ.value
      },
      cacheRadius: chunkCacheRadius.value,
      batchSize: chunkBatchSize.value,
      chunks: (packetById.get(45) ?? []).map(packet => ({
        x: packet.body.readInt32BE(0),
        z: packet.body.readInt32BE(4)
      })),
      chunkBiomePalettes: (packetById.get(45) ?? []).map(packet => decodeLevelChunkBiomePalette(packet, registryPackets))
    }
  }
  if (recordOnly) {
    const packetById = new Map()
    for (const packet of playPackets) {
      if (!packetById.has(packet.id)) packetById.set(packet.id, [])
      packetById.get(packet.id).push(packet)
    }
    const positionPacket = packetById.get(72)?.[0]
    if (positionPacket) {
      joinState.position = decodePlayerPositionPacket(positionPacket.body)
    }
    const chunkCacheRadiusPacket = packetById.get(95)?.[0]
    const chunkCacheRadius = chunkCacheRadiusPacket && readVarInt(chunkCacheRadiusPacket.body)
    const chunkCacheCenterPacket = packetById.get(94)?.[0]
    const chunkCacheCenterX = chunkCacheCenterPacket && readVarInt(chunkCacheCenterPacket.body)
    const chunkCacheCenterZ = chunkCacheCenterX && readVarInt(chunkCacheCenterPacket.body, chunkCacheCenterX.offset)
    const chunkBatchFinishedPacket = packetById.get(11)?.[0]
    const chunkBatchSize = chunkBatchFinishedPacket && readVarInt(chunkBatchFinishedPacket.body)
    if (chunkCacheRadius && chunkCacheCenterX && chunkCacheCenterZ && chunkBatchSize) {
      joinState.chunkStreaming = {
        cacheCenter: { x: chunkCacheCenterX.value, z: chunkCacheCenterZ.value },
        cacheRadius: chunkCacheRadius.value,
        batchSize: chunkBatchSize.value,
        chunks: (packetById.get(45) ?? []).map(packet => ({
          x: packet.body.readInt32BE(0),
          z: packet.body.readInt32BE(4)
        })),
        chunkBiomePalettes: (packetById.get(45) ?? []).map(packet => decodeLevelChunkBiomePalette(packet, registryPackets))
      }
    }
  }
  joinState.lastReceivedChunk = play.filter(packet => packet.id === 45).length === 0
    ? null
    : play.filter(packet => packet.id === 45).length - 1
  joinState.initialChunkCount = play.filter(packet => packet.id === 45).length
  socket.write(encodeClientPacket(reader, serverboundAcceptTeleportationPacketId, writeVarInt(0)))
  socket.write(encodeClientPacket(reader, serverboundChunkBatchReceivedPacketId, Buffer.alloc(4)))
  socket.write(encodeClientPacket(reader, serverboundPlayerLoadedPacketId))
  if (firstTickActions.size > 0) {
    if (firstTickActions.has('client_information')) socket.write(encodeClientPacket(reader, serverboundClientInformationPacketId, clientInformationPayload()))
    if (firstTickActions.has('held_slot')) socket.write(encodeClientPacket(reader, serverboundSetCarriedItemPacketId, writeShort(carriedItemSlot)))
    if (firstTickActions.has('movement')) socket.write(encodeClientPacket(reader, serverboundMovePlayerPosRotPacketId, movePlayerPosRotPayload()))
    if (firstTickActions.has('chat')) socket.write(encodeClientPacket(reader, serverboundChatPacketId, chatPayload('first tick')))
    if (firstTickActions.has('command_suggestion')) socket.write(encodeClientPacket(reader, serverboundCommandSuggestionPacketId, commandSuggestionPayload('/list')))
    if (firstTickActions.has('inventory_click')) socket.write(encodeClientPacket(reader, serverboundContainerClickPacketId, containerClickPayload()))
    if (firstTickActions.has('inventory_close')) socket.write(encodeClientPacket(reader, serverboundContainerClosePacketId, Buffer.from([0])))
    if (firstTickActions.has('block_action')) socket.write(encodeClientPacket(reader, serverboundPlayerActionPacketId, playerActionPayload()))
    if (firstTickActions.has('player_input')) socket.write(encodeClientPacket(reader, serverboundPlayerInputPacketId, playerInputPayload()))
    if (firstTickActions.has('swing')) socket.write(encodeClientPacket(reader, serverboundSwingPacketId, writeVarInt(0)))
    if (firstTickActions.has('use_item_on')) socket.write(encodeClientPacket(reader, serverboundUseItemOnPacketId, useItemOnPayload()))
    if (firstTickActions.has('use_item')) socket.write(encodeClientPacket(reader, serverboundUseItemPacketId, useItemPayload()))
    if (abortAfter === 'first_tick_actions') return abortSocket(socket, 'first_tick_actions', { login: login.id, config, play, joinState })
  }
  for (const position of extraMovementPositions) {
    socket.write(encodeClientPacket(reader, serverboundMovePlayerPosRotPacketId, movePlayerPosRotPayload(position)))
  }

  if (postActionProbeMs > 0) {
    const deadline = Date.now() + postActionProbeMs
    while (Date.now() < deadline) {
      const waitMs = Math.max(1, deadline - Date.now())
      const next = await nextPacketWithin(reader, waitMs)
      if (next.timeout) break
      const packet = next.packet
      if (packet.id === clientboundKeepAlivePacketId) {
        if (packet.body.length !== 8) {
          throw new Error(`expected 8-byte keep_alive payload, got ${packet.body.length}`)
        }
        keepAliveReplies += 1
        socket.write(encodeClientPacket(reader, serverboundKeepAlivePacketId, packet.body))
      }
      play.push({ id: packet.id, length: packet.length })
      playPackets.push(packet)
    }
    const initialBatchEnd = playPackets.findIndex(packet => packet.id === 11)
    const dynamicPackets = initialBatchEnd === -1 ? [] : playPackets.slice(initialBatchEnd + 1)
    const batches = parseDynamicChunkBatches(dynamicPackets)
    joinState.dynamicChunkStreamingBatches = batches
    joinState.dynamicChunkStreaming = batches.at(-1)
  }

  let commandSuggestionSeen = false
  if (keepAliveProbeMs > 0) {
    const deadline = Date.now() + keepAliveProbeMs
    while (Date.now() < deadline) {
      const waitMs = Math.max(1, deadline - Date.now())
      const next = await nextPacketWithin(reader, waitMs)
      if (next.timeout) break
      const packet = next.packet
      if (packet.id !== clientboundKeepAlivePacketId) {
        if (packet.id === clientboundCommandSuggestionsPacketId) {
          commandSuggestionSeen ||= commandSuggestionMatches(packet.body, expectedCommandSuggestion)
        }
        play.push({ id: packet.id, length: packet.length })
        continue
      }
      if (packet.body.length !== 8) {
        throw new Error(`expected 8-byte keep_alive payload, got ${packet.body.length}`)
      }
      keepAliveReplies += 1
      play.push({ id: packet.id, length: packet.length })
      if (abortAfter === 'first_keepalive') return abortSocket(socket, 'first_keepalive', { login: login.id, config, play, joinState, keepAliveReplies })
      socket.write(encodeClientPacket(reader, serverboundKeepAlivePacketId, packet.body))
    }
    if (!recordOnly && keepAliveReplies === 0) {
      throw new Error(`no clientbound keep_alive observed within ${keepAliveProbeMs}ms`)
    }
  }
  if (expectedCommandSuggestion && !commandSuggestionSeen) {
    throw new Error(`missing command suggestion ${expectedCommandSuggestion}`)
  }

  socket.end()
  const result = { ok: true, mode: recordOnly ? 'record' : 'strict', host, port, login: login.id, compressionThreshold, config, play, joinState, keepAliveReplies }
  if (summaryOnly) {
    console.log(JSON.stringify({
      ok: result.ok,
      mode: result.mode,
      host: result.host,
      port: result.port,
      login: result.login,
      compressionThreshold: result.compressionThreshold,
      playPacketCount: result.play.length,
      configPacketCount: result.config.length,
      keepAliveReplies: result.keepAliveReplies,
      joinState: {
        profile: result.joinState.profile,
        position: result.joinState.position,
        initialChunkCount: result.joinState.initialChunkCount,
        lastReceivedChunk: result.joinState.lastReceivedChunk
      }
    }, null, 2))
    return
  }
  console.log(JSON.stringify(result, null, 2))
}

function abortSocket (socket, phase, details) {
  socket.destroy()
  if (summaryOnly) {
    console.log(JSON.stringify({
      ok: true,
      aborted: true,
      phase,
      login: details.login,
      compressionThreshold: details.compressionThreshold,
      configPacketCount: details.config?.length ?? 0,
      playPacketCount: details.play?.length ?? 0,
      joinState: details.joinState == null
        ? undefined
        : {
            profile: details.joinState.profile,
            position: details.joinState.position,
            initialChunkCount: details.joinState.initialChunkCount,
            lastReceivedChunk: details.joinState.lastReceivedChunk
          },
      keepAliveReplies: details.keepAliveReplies
    }, null, 2))
    return
  }
  console.log(JSON.stringify({ ok: true, aborted: true, phase, ...details }, null, 2))
}

function readLoginSpawnInfo (body) {
  let offset = 5
  const levelCount = readVarInt(body, offset); offset = levelCount.offset
  for (let i = 0; i < levelCount.value; i++) {
    const level = readString(body, offset); offset = level.offset
  }
  const maxPlayers = readVarInt(body, offset); offset = maxPlayers.offset
  const chunkRadius = readVarInt(body, offset); offset = chunkRadius.offset
  const simulationDistance = readVarInt(body, offset); offset = simulationDistance.offset
  offset += 3
  const dimensionType = readVarInt(body, offset); offset = dimensionType.offset
  const dimension = readString(body, offset); offset = dimension.offset
  const seed = body.readBigInt64BE(offset); offset += 8
  const gameMode = body[offset++]
  const previousGameMode = body[offset++]
  const isDebug = body[offset++] === 1
  const isFlat = body[offset++] === 1
  const lastDeathPresent = body[offset++] === 1
  if (lastDeathPresent) {
    const lastDeathDimension = readString(body, offset); offset = lastDeathDimension.offset
    offset += 12
  }
  const portalCooldown = readVarInt(body, offset); offset = portalCooldown.offset
  const seaLevel = readVarInt(body, offset); offset = seaLevel.offset
  return {
    seed,
    gameMode,
    previousGameMode,
    isDebug,
    isFlat,
    chunkRadius: chunkRadius.value,
    simulationDistance: simulationDistance.value,
    portalCooldown: portalCooldown.value,
    seaLevel: seaLevel.value
  }
}

function decodePlayerPositionPacket (body) {
  let offset = 0
  const teleportId = readVarInt(body, offset)
  offset = teleportId.offset
  const x = body.readDoubleBE(offset); offset += 8
  const y = body.readDoubleBE(offset); offset += 8
  const z = body.readDoubleBE(offset); offset += 8
  offset += 24
  const yaw = body.readFloatBE(offset); offset += 4
  const pitch = body.readFloatBE(offset); offset += 4
  const relatives = body.length - offset >= 4
    ? body.readInt32BE(offset)
    : body.readUInt8(offset)
  return { teleportId: teleportId.value, x, y, z, yaw, pitch, relatives }
}

function decodeLevelChunkBiomePalette (packet, registryPackets) {
  const biomeRegistry = registryPackets.find(packet => packet.registry === 'minecraft:worldgen/biome')?.elementIds ?? []
  let offset = 0
  const x = packet.body.readInt32BE(offset); offset += 4
  const z = packet.body.readInt32BE(offset); offset += 4
  const heightmapCount = readVarInt(packet.body, offset); offset = heightmapCount.offset
  for (let i = 0; i < heightmapCount.value; i++) {
    const heightmapType = readVarInt(packet.body, offset); offset = heightmapType.offset
    const length = readVarInt(packet.body, offset); offset = length.offset + length.value * 8
  }
  const sectionDataLength = readVarInt(packet.body, offset); offset = sectionDataLength.offset
  const sectionDataEnd = offset + sectionDataLength.value
  const biomePaletteIds = new Set()
  let sectionCount = 0
  while (offset < sectionDataEnd) {
    const section = decodeNetworkChunkSection(packet.body, offset)
    offset = section.offset
    sectionCount += 1
    for (const id of section.biomePaletteIds) biomePaletteIds.add(id)
  }
  if (offset !== sectionDataEnd) {
    throw new Error(`chunk ${x},${z} section data parser stopped ${offset - sectionDataEnd} bytes from section end`)
  }
  const blockEntityCount = readVarInt(packet.body, offset)
  if (!blockEntityCount) throw new Error(`chunk ${x},${z} missing block entity count`)
  return {
    x,
    z,
    sectionCount,
    biomePaletteIds: [...biomePaletteIds].sort((left, right) => left - right),
    biomePalette: [...biomePaletteIds]
      .sort((left, right) => left - right)
      .map(id => biomeRegistry[id] ?? `unknown:${id}`)
  }
}

function decodeNetworkChunkSection (buffer, offset) {
  offset += 2 // nonEmptyBlockCount
  offset += 2 // fluidCount
  const blocks = decodeNetworkPalettedContainer(buffer, offset, 4096)
  const biomes = decodeNetworkPalettedContainer(buffer, blocks.offset, 64)
  return {
    offset: biomes.offset,
    biomePaletteIds: biomes.paletteIds
  }
}

function decodeNetworkPalettedContainer (buffer, offset, entries) {
  const bitsPerEntry = buffer[offset++]
  const paletteIds = []
  if (bitsPerEntry === 0) {
    const single = readVarInt(buffer, offset)
    if (!single) throw new Error('missing single-value paletted container id')
    offset = single.offset
    paletteIds.push(single.value)
  } else {
    const paletteLength = readVarInt(buffer, offset)
    if (!paletteLength) throw new Error('missing paletted container palette length')
    offset = paletteLength.offset
    for (let i = 0; i < paletteLength.value; i++) {
      const id = readVarInt(buffer, offset)
      if (!id) throw new Error('missing paletted container palette id')
      offset = id.offset
      paletteIds.push(id.value)
    }
    offset += Math.ceil(entries * bitsPerEntry / 64) * 8
  }
  return { offset, paletteIds }
}

function commandSuggestionMatches (body, expected) {
  if (!expected) return true
  const transaction = readVarInt(body, 0)
  let offset = transaction.offset
  const start = readVarInt(body, offset); offset = start.offset
  const length = readVarInt(body, offset); offset = length.offset
  const count = readVarInt(body, offset); offset = count.offset
  for (let i = 0; i < count.value; i++) {
    const match = readString(body, offset); offset = match.offset
    const hasTooltip = body[offset++] === 1
    if (hasTooltip) {
      const tooltip = readString(body, offset)
      offset = tooltip.offset
    }
    if (match.value === expected) return true
  }
  return false
}

function clientInformationPayload () {
  return Buffer.concat([
    writeString('en_us'),
    Buffer.from([10]),
    writeVarInt(0),
    Buffer.from([1, 0x7f]),
    writeVarInt(1),
    Buffer.from([0, 1]),
    writeVarInt(0)
  ])
}

function movePlayerPosRotPayload (position = movementPosition) {
  const payload = Buffer.alloc(33)
  payload.writeDoubleBE(position.x, 0)
  payload.writeDoubleBE(position.y, 8)
  payload.writeDoubleBE(position.z, 16)
  payload.writeFloatBE(position.yaw, 24)
  payload.writeFloatBE(position.pitch, 28)
  payload.writeUInt8(1, 32)
  return payload
}

function parsePositionEnv (value, fallback) {
  if (!value) return fallback
  const parsed = JSON.parse(value)
  return parsePosition(parsed)
}

function parseBlockPosEnv (value, fallback) {
  if (!value) return fallback
  const parsed = JSON.parse(value)
  return {
    x: Number(parsed.x),
    y: Number(parsed.y),
    z: Number(parsed.z)
  }
}

function parsePositionArrayEnv (value) {
  if (!value) return []
  const parsed = JSON.parse(value)
  if (!Array.isArray(parsed)) throw new Error('VIBECRAFT_RAW_PROBE_EXTRA_MOVEMENTS must be a JSON array')
  return parsed.map(parsePosition)
}

function parsePosition (parsed) {
  return {
    x: Number(parsed.x),
    y: Number(parsed.y),
    z: Number(parsed.z),
    yaw: Number(parsed.yaw ?? 0),
    pitch: Number(parsed.pitch ?? 0)
  }
}

function parseDynamicChunkBatches (packets) {
  const batches = []
  let pendingForgottenChunks = []
  let current = null
  for (const packet of packets) {
    if (packet.id === 37) {
      pendingForgottenChunks.push({
        x: packet.body.readInt32BE(4),
        z: packet.body.readInt32BE(0)
      })
    } else if (packet.id === 94) {
      const x = readVarInt(packet.body)
      const z = x && readVarInt(packet.body, x.offset)
      if (!x || !z) throw new Error('malformed dynamic chunk cache center')
      current = {
        cacheCenter: { x: x.value, z: z.value },
        batchSize: null,
        chunks: [],
        forgottenChunks: pendingForgottenChunks
      }
      pendingForgottenChunks = []
      batches.push(current)
    } else if (packet.id === 45 && current) {
      current.chunks.push({
        x: packet.body.readInt32BE(0),
        z: packet.body.readInt32BE(4)
      })
    } else if (packet.id === 11 && current) {
      const size = readVarInt(packet.body)
      if (!size) throw new Error('malformed dynamic chunk batch finished')
      current.batchSize = size.value
    }
  }
  return batches
}

function chatPayload (message) {
  return Buffer.concat([
    writeString(message),
    Buffer.alloc(8),
    Buffer.alloc(8),
    Buffer.from([0]),
    writeVarInt(0),
    Buffer.from([0])
  ])
}

function commandSuggestionPayload (command) {
  return Buffer.concat([writeVarInt(1), writeString(command)])
}

function containerClickPayload () {
  const slot = Buffer.alloc(2)
  slot.writeInt16BE(0, 0)
  return Buffer.concat([
    Buffer.from([0]),
    writeVarInt(0),
    slot,
    Buffer.from([0]),
    writeVarInt(0),
    writeVarInt(0),
    Buffer.from([0])
  ])
}

function playerInputPayload () {
  return Buffer.from([0x09])
}

function playerActionPayload () {
  const sequence = writeVarInt(1)
  const payload = Buffer.alloc(10 + sequence.length)
  // START_DESTROY_BLOCK at the block under the player, facing up.
  payload[0] = 0
  payload.writeBigInt64BE(80n, 1)
  payload[9] = 1
  sequence.copy(payload, 10)
  return payload
}

function useItemOnPayload () {
  const sequence = writeVarInt(2)
  const payload = Buffer.alloc(22 + sequence.length)
  let offset = 0
  writeVarInt(0).copy(payload, offset)
  offset += 1
  payload.writeBigInt64BE(80n, offset)
  offset += 8
  writeVarInt(1).copy(payload, offset)
  offset += 1
  payload.writeFloatBE(0.5, offset)
  payload.writeFloatBE(1.0, offset + 4)
  payload.writeFloatBE(0.5, offset + 8)
  offset += 12
  payload[offset++] = 0
  payload[offset++] = 0
  sequence.copy(payload, offset)
  return payload
}

function useItemPayload () {
  const sequence = writeVarInt(3)
  const payload = Buffer.alloc(10 + sequence.length)
  let offset = 0
  writeVarInt(0).copy(payload, offset)
  offset += 1
  sequence.copy(payload, offset)
  offset += sequence.length
  payload.writeFloatBE(0, offset)
  payload.writeFloatBE(0, offset + 4)
  return payload
}

main().catch(error => {
  console.error(JSON.stringify({ ok: false, error: error.message, stack: error.stack }, null, 2))
  process.exit(1)
})
