import net from 'node:net'
import crypto from 'node:crypto'

const host = process.env.RUSTCRAFT_HOST ?? '127.0.0.1'
const port = Number(process.env.RUSTCRAFT_PORT ?? 25565)
const username = process.env.RUSTCRAFT_USERNAME ?? 'RustCraftProbe'
const protocolVersion = 775
const serverboundAcceptTeleportationPacketId = 0
const serverboundPlayerLoadedPacketId = 44

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

function expectPacket (packet, id, state) {
  if (packet.id !== id) {
    throw new Error(`expected ${state} packet ${id}, got ${packet.id}`)
  }
}

const expectedRegistries = [
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
  ['minecraft:chat_type', 7],
  ['minecraft:cat_variant', 11],
  ['minecraft:banner_pattern', 43],
  ['minecraft:chicken_variant', 3],
  ['minecraft:cow_variant', 3],
  ['minecraft:frog_variant', 3],
  ['minecraft:instrument', 8],
  ['minecraft:jukebox_song', 21],
  ['minecraft:pig_variant', 3],
  ['minecraft:trim_pattern', 18],
  ['minecraft:wolf_variant', 9]
])

const requiredRegistryElements = new Map([
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
  ['minecraft:cat_sound_variant', ['minecraft:default']],
  ['minecraft:chicken_variant', [
    'minecraft:cold',
    'minecraft:temperate',
    'minecraft:warm'
  ]],
  ['minecraft:chicken_sound_variant', ['minecraft:default']],
  ['minecraft:cow_sound_variant', ['minecraft:default']],
  ['minecraft:damage_type', ['minecraft:spear']],
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
  ['minecraft:pig_sound_variant', ['minecraft:default']],
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
  ['minecraft:wolf_sound_variant', ['minecraft:default']],
  ['minecraft:zombie_nautilus_variant', ['minecraft:default']]
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

function decodeRegistryPacket (packet) {
  const registry = readString(packet.body)
  const count = readVarInt(packet.body, registry.offset)
  if (!count) throw new Error(`missing element count for registry ${registry.value}`)
  let offset = count.offset
  const elements = []
  for (let i = 0; i < count.value; i++) {
    const element = readString(packet.body, offset)
    elements.push(element.value)
    offset = element.offset
    const hasData = packet.body[offset++]
    if (hasData === 1) {
      offset = skipNetworkNbt(packet.body, offset)
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
    elementIds: elements
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

function skipNetworkNbt (buffer, offset) {
  const type = buffer[offset++]
  if (type !== 10) throw new Error(`expected compound NBT tag, got ${type}`)
  return skipNbtPayload(buffer, offset, type)
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
    } else {
      config.push({ id: packet.id, length: packet.length })
    }
    if (packet.id === 3) break
  }
  const configIds = config.map(packet => packet.id)
  for (const id of [12, 7, 13, 3]) {
    if (!configIds.includes(id)) throw new Error(`missing configuration packet ${id}`)
  }
  const registryPackets = config.filter(packet => packet.id === 7)
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
  socket.write(frame(3))

  const play = []
  for (let i = 0; i < 6; i++) {
    const packet = await reader.nextPacket()
    play.push({ id: packet.id, length: packet.length })
  }
  for (const id of [49, 105, 72, 12, 48, 11]) {
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
  socket.write(frame(serverboundAcceptTeleportationPacketId, writeVarInt(0)))
  socket.write(frame(serverboundPlayerLoadedPacketId))

  socket.end()
  console.log(JSON.stringify({ ok: true, host, port, login: login.id, config, play }, null, 2))
}

main().catch(error => {
  console.error(JSON.stringify({ ok: false, error: error.message, stack: error.stack }, null, 2))
  process.exit(1)
})
