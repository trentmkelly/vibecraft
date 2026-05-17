import net from 'node:net'
import crypto from 'node:crypto'

const host = process.env.RUSTCRAFT_HOST ?? '127.0.0.1'
const port = Number(process.env.RUSTCRAFT_PORT ?? 25565)
const username = process.env.RUSTCRAFT_USERNAME ?? 'RustCraftProbe'
const protocolVersion = 775
const recordOnly = process.env.RUSTCRAFT_RAW_PROBE_MODE === 'record'
const keepAliveProbeMs = Number(process.env.RUSTCRAFT_RAW_PROBE_KEEPALIVE_MS ?? 0)
const serverboundAcceptTeleportationPacketId = 0
const serverboundKeepAlivePacketId = 28
const serverboundSelectKnownPacksPacketId = 7
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

function readString (buffer, offset = 0) {
  const length = readVarInt(buffer, offset)
  if (!length) throw new Error('missing string length')
  const end = length.offset + length.value
  if (buffer.length < end) throw new Error('truncated string')
  return { value: buffer.subarray(length.offset, end).toString('utf8'), offset: end }
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
      const packetId = readVarInt(payload)
      if (!packetId) throw new Error('packet missing id')
      this.waiters.shift().resolve({
        id: packetId.value,
        body: payload.subarray(packetId.offset),
        length: length.value
      })
    }
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
  'minecraft:zombie_nautilus_variant'
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
  ['minecraft:trim_material', 11],
  ['minecraft:trim_pattern', 18],
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

  const login = await reader.nextPacket()
  expectPacket(login, 2, 'login_finished')
  socket.write(frame(3))

  const config = []
  while (true) {
    const packet = await reader.nextPacket()
    if (packet.id === 7) {
      config.push(decodeRegistryPacket(packet))
    } else if (packet.id === 13) {
      config.push(decodeTagsPacket(packet))
    } else if (packet.id === 14) {
      const knownPacks = decodeKnownPacksPacket(packet)
      config.push(knownPacks)
      socket.write(frame(
        serverboundSelectKnownPacksPacketId,
        writeVarInt(knownPacks.packs.length),
        ...knownPacks.packs.map(writeKnownPack)
      ))
    } else {
      config.push({ id: packet.id, length: packet.length })
    }
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
  socket.write(frame(3))

  const play = []
  const expectedPlayPacketIds = [49, 10, 64, 105, 72, 43, 97, 94, 95, 38, 12, 45, 11]
  for (let i = 0; i < expectedPlayPacketIds.length; i++) {
    const packet = await reader.nextPacket()
    play.push({ id: packet.id, length: packet.length })
  }
  if (!recordOnly) {
  for (const id of expectedPlayPacketIds) {
      if (!play.some(packet => packet.id === id)) throw new Error(`missing play packet ${id}`)
    }
    const loginPacket = play.find(packet => packet.id === 49)
    if (!loginPacket || loginPacket.length !== 70) {
      throw new Error(`expected 70-byte play login packet after holder-id encoding, got ${loginPacket?.length}`)
    }
    const positionPacket = play.find(packet => packet.id === 72)
    if (!positionPacket || positionPacket.length !== 62) {
      throw new Error(`expected 62-byte player_position packet with fixed-int relatives, got ${positionPacket?.length}`)
    }
  }
  socket.write(frame(serverboundAcceptTeleportationPacketId, writeVarInt(0)))
  socket.write(frame(serverboundPlayerLoadedPacketId))

  let keepAliveReplies = 0
  if (keepAliveProbeMs > 0) {
    const deadline = Date.now() + keepAliveProbeMs
    while (Date.now() < deadline) {
      const waitMs = Math.max(1, deadline - Date.now())
      const next = await nextPacketWithin(reader, waitMs)
      if (next.timeout) break
      const packet = next.packet
      if (packet.id !== clientboundKeepAlivePacketId) {
        play.push({ id: packet.id, length: packet.length })
        continue
      }
      if (packet.body.length !== 8) {
        throw new Error(`expected 8-byte keep_alive payload, got ${packet.body.length}`)
      }
      keepAliveReplies += 1
      play.push({ id: packet.id, length: packet.length })
      socket.write(frame(serverboundKeepAlivePacketId, packet.body))
    }
    if (!recordOnly && keepAliveReplies === 0) {
      throw new Error(`no clientbound keep_alive observed within ${keepAliveProbeMs}ms`)
    }
  }

  socket.end()
  console.log(JSON.stringify({ ok: true, mode: recordOnly ? 'record' : 'strict', host, port, login: login.id, config, play, keepAliveReplies }, null, 2))
}

main().catch(error => {
  console.error(JSON.stringify({ ok: false, error: error.message, stack: error.stack }, null, 2))
  process.exit(1)
})
