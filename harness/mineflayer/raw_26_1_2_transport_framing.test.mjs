import assert from 'node:assert/strict'
import crypto from 'node:crypto'
import net from 'node:net'
import test from 'node:test'
import { inflateSync } from 'node:zlib'

const host = process.env.VIBECRAFT_HOST ?? '127.0.0.1'
const port = Number(process.env.VIBECRAFT_PORT ?? 25565)
const protocolVersion = Number(process.env.VIBECRAFT_PROTOCOL_VERSION ?? 775)

test('raw 26.1.2 login transport framing preserves packet boundaries through configuration entry', async () => {
  const username = 'FrameProbe'
  const socket = await connect()
  const reader = new FrameReader(socket)

  const handshakeFrame = frame(0, handshakePayload())
  const loginStartFrame = frame(0, writeString(username), randomUuidBytes())
  assert.deepEqual(decodeFrame(handshakeFrame).packetId, 0)
  assert.deepEqual(decodeFrame(loginStartFrame).packetId, 0)
  socket.write(handshakeFrame)
  socket.write(loginStartFrame)

  let compressionThreshold = null
  let loginSuccess = await reader.nextFrame()
  if (loginSuccess.packetId === 3) {
    const threshold = readVarInt(loginSuccess.body, 0)
    assert.ok(threshold)
    compressionThreshold = threshold.value
    reader.setCompression(compressionThreshold)
    loginSuccess = await reader.nextFrame()
  }
  assert.equal(loginSuccess.packetId, 2)
  assert.equal(loginSuccess.body.length, 16 + 1 + username.length + 1)
  assert.equal(readUuid(loginSuccess.body, 0), offlineUuid(username))
  const name = readString(loginSuccess.body, 16)
  assert.equal(name.value, username)
  const properties = readVarInt(loginSuccess.body, name.offset)
  assert.equal(properties.value, 0)
  assert.equal(properties.offset, loginSuccess.body.length)

  socket.write(encodeClientPacket(compressionThreshold, 3))
  const firstConfig = await reader.nextFrame()
  assert.equal(firstConfig.packetId, 12)
  const featureCount = readVarInt(firstConfig.body, 0)
  assert.equal(featureCount.value, 1)
  const feature = readString(firstConfig.body, featureCount.offset)
  assert.equal(feature.value, 'minecraft:vanilla')
  assert.equal(feature.offset, firstConfig.body.length)

  const firstRegistry = await reader.nextFrame()
  assert.equal(firstRegistry.packetId, 7)
  assert.equal(readString(firstRegistry.body, 0).value, 'minecraft:worldgen/biome')

  socket.destroy()
})

test('raw 26.1.2 login transport framing preserves unsupported-protocol disconnect boundary', async () => {
  const username = 'FrameReject'
  const socket = await connect()
  const reader = new FrameReader(socket)

  socket.write(frame(0, handshakePayload({ protocol: 1 })))
  socket.write(frame(0, writeString(username), randomUuidBytes()))

  const disconnect = await reader.nextFrame()
  assert.equal(disconnect.packetId, 0)
  assert.equal(disconnect.payload.length, disconnect.length)
  assert.ok(disconnect.body.length > 0)
  assert.equal(reader.buffer.length, 0)
  socket.destroy()
})

class FrameReader {
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

  nextFrame () {
    return new Promise((resolve, reject) => {
      this.waiters.push({ resolve, reject })
      this.pump()
    })
  }

  pump () {
    while (this.waiters.length > 0) {
      const decoded = tryDecodeFrame(this.buffer, this.compressionThreshold)
      if (!decoded) return
      this.buffer = this.buffer.subarray(decoded.frameLength)
      this.waiters.shift().resolve(decoded.frame)
    }
  }

  setCompression (threshold) {
    this.compressionThreshold = threshold
  }

  rejectAll (error) {
    for (const waiter of this.waiters.splice(0)) waiter.reject(error)
  }
}

async function connect () {
  const socket = net.createConnection({ host, port })
  await new Promise((resolve, reject) => {
    socket.once('connect', resolve)
    socket.once('error', reject)
  })
  return socket
}

function tryDecodeFrame (buffer, compressionThreshold = null) {
  const length = readVarInt(buffer)
  if (!length) return null
  const end = length.offset + length.value
  if (buffer.length < end) return null
  let payload = buffer.subarray(length.offset, end)
  if (compressionThreshold != null) {
    const dataLength = readVarInt(payload)
    if (!dataLength) return null
    const body = payload.subarray(dataLength.offset)
    if (dataLength.value > 0) {
      payload = inflateSync(body)
      assert.equal(payload.length, dataLength.value)
    } else {
      payload = body
    }
  }
  const packetId = readVarInt(payload)
  return {
    frameLength: end,
    frame: {
      length: length.value,
      packetId: packetId.value,
      payload,
      body: payload.subarray(packetId.offset)
    }
  }
}

function decodeFrame (buffer) {
  const decoded = tryDecodeFrame(buffer)
  if (!decoded || decoded.frameLength !== buffer.length) throw new Error('invalid frame')
  return decoded.frame
}

function handshakePayload ({ protocol = protocolVersion } = {}) {
  return Buffer.concat([
    writeVarInt(protocol),
    writeString(host),
    Buffer.from([(port >> 8) & 0xff, port & 0xff]),
    writeVarInt(2)
  ])
}

function frame (packetId, ...parts) {
  const payload = Buffer.concat([writeVarInt(packetId), ...parts])
  return Buffer.concat([writeVarInt(payload.length), payload])
}

function encodeClientPacket (compressionThreshold, packetId, ...parts) {
  const payload = Buffer.concat([writeVarInt(packetId), ...parts])
  if (compressionThreshold == null) return Buffer.concat([writeVarInt(payload.length), payload])
  const framedPayload = Buffer.concat([writeVarInt(0), payload])
  return Buffer.concat([writeVarInt(framedPayload.length), framedPayload])
}

function writeString (value) {
  const data = Buffer.from(value, 'utf8')
  return Buffer.concat([writeVarInt(data.length), data])
}

function readString (buffer, offset = 0) {
  const length = readVarInt(buffer, offset)
  if (!length) throw new Error('missing string length')
  const end = length.offset + length.value
  return { value: buffer.subarray(length.offset, end).toString('utf8'), offset: end }
}

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

function randomUuidBytes () {
  return Buffer.from(crypto.randomUUID().replaceAll('-', ''), 'hex')
}

function offlineUuid (name) {
  const hash = crypto.createHash('md5').update(`OfflinePlayer:${name}`, 'utf8').digest()
  hash[6] = (hash[6] & 0x0f) | 0x30
  hash[8] = (hash[8] & 0x3f) | 0x80
  const hex = hash.toString('hex')
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`
}

function readUuid (buffer, offset) {
  const hex = buffer.subarray(offset, offset + 16).toString('hex')
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`
}
