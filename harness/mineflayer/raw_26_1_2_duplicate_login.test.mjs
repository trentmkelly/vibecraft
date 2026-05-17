import assert from 'node:assert/strict'
import crypto from 'node:crypto'
import net from 'node:net'
import test from 'node:test'
import { inflateSync } from 'node:zlib'

const host = process.env.RUSTCRAFT_HOST ?? '127.0.0.1'
const port = Number(process.env.RUSTCRAFT_PORT ?? 25565)
const protocolVersion = Number(process.env.RUSTCRAFT_PROTOCOL_VERSION ?? 775)
const username = process.env.RUSTCRAFT_DUPLICATE_USERNAME ?? 'DupLoginProbe'

test('raw 26.1.2 duplicate offline login is rejected while the first session is active', { timeout: 30_000 }, async () => {
  const first = await connect()
  const firstReader = new FrameReader(first)
  try {
    await enterPlay(first, firstReader, username)

    const second = await connect()
    const secondReader = new FrameReader(second)
    try {
      socketLoginHello(second, username)
      const disconnect = await secondReader.nextFrame()
      assert.equal(disconnect.packetId, 0, 'duplicate login should receive login disconnect')
      const reason = readString(disconnect.body, 0)
      assert.match(reason.value, /multiplayer\.disconnect\.duplicate_login/)
    } finally {
      second.destroy()
    }

    first.destroy()
    await delay(100)

    const retry = await connect()
    const retryReader = new FrameReader(retry)
    try {
      await enterPlay(retry, retryReader, username)
    } finally {
      retry.destroy()
    }
  } finally {
    first.destroy()
  }
})

async function enterPlay (socket, reader, name) {
  socketLoginHello(socket, name)
  let loginSuccess = await reader.nextFrame()
  if (loginSuccess.packetId === 3) {
    const threshold = readVarInt(loginSuccess.body, 0)
    if (!threshold) throw new Error('missing compression threshold')
    reader.setCompression(threshold.value)
    loginSuccess = await reader.nextFrame()
  }
  assert.equal(loginSuccess.packetId, 2)
  socket.write(encodeClientPacket(reader.compressionThreshold, 3))
  await waitForPacket(reader, 14)
  socket.write(encodeClientPacket(reader.compressionThreshold, 7, writeVarInt(0)))
  await waitForPacket(reader, 3)
  socket.write(encodeClientPacket(reader.compressionThreshold, 3))
  await waitForPacket(reader, 49)
}

function socketLoginHello (socket, name) {
  socket.write(frame(0, handshakePayload(2)))
  socket.write(frame(0, writeString(name), randomUuidBytes()))
}

async function waitForPacket (reader, packetId) {
  const deadline = Date.now() + 10_000
  while (Date.now() < deadline) {
    const packet = await reader.nextFrame()
    if (packet.packetId === packetId) return packet
  }
  throw new Error(`timed out waiting for packet ${packetId}`)
}

class FrameReader {
  constructor (socket) {
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

  rejectAll (error) {
    for (const waiter of this.waiters.splice(0)) waiter.reject(error)
  }

  setCompression (threshold) {
    this.compressionThreshold = threshold
  }
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
    payload = dataLength.value > 0 ? inflateSync(body) : body
  }
  const packetId = readVarInt(payload)
  return {
    frameLength: end,
    frame: {
      length: length.value,
      packetId: packetId.value,
      body: payload.subarray(packetId.offset)
    }
  }
}

async function connect () {
  const socket = net.createConnection({ host, port })
  socket.setMaxListeners(64)
  await Promise.race([
    once(socket, 'connect'),
    once(socket, 'error').then(error => Promise.reject(error))
  ])
  return socket
}

function handshakePayload (nextState) {
  return Buffer.concat([
    writeVarInt(protocolVersion),
    writeString(host),
    Buffer.from([(port >> 8) & 0xff, port & 0xff]),
    writeVarInt(nextState)
  ])
}

function frame (packetId, ...parts) {
  const payload = Buffer.concat([writeVarInt(packetId), ...parts])
  return Buffer.concat([writeVarInt(payload.length), payload])
}

function encodeClientPacket (compressionThreshold, packetId, ...parts) {
  const payload = Buffer.concat([writeVarInt(packetId), ...parts])
  if (compressionThreshold == null) return Buffer.concat([writeVarInt(payload.length), payload])
  const compressedPayload = Buffer.concat([writeVarInt(0), payload])
  return Buffer.concat([writeVarInt(compressedPayload.length), compressedPayload])
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

function once (emitter, event) {
  return new Promise(resolve => emitter.once(event, resolve))
}

function delay (ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}
