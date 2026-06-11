import assert from 'node:assert/strict'
import crypto from 'node:crypto'
import { createConnection, createServer } from 'node:net'
import { rm } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { inflateSync } from 'node:zlib'

import {
  createTempWorld,
  startVibeCraft,
  stopServer,
  waitForPort,
  writeOfflineServerFiles
} from './runner.mjs'

const here = new URL('.', import.meta.url)
const repoRoot = path.resolve(here.pathname, '..', '..')
const binary = path.join(repoRoot, 'target', 'debug', 'vibecraft')
const host = '127.0.0.1'
const protocolVersion = 775
let port = 0

test('raw 26.1.2 login state machine rejects play packets before play entry', { timeout: 30_000 }, async () => {
  port = await reservePort()
  const root = await createTempWorld('vibecraft-state-machine-')
  let server

  try {
    await writeOfflineServerFiles(root, {
      port,
      levelName: 'world',
      properties: {
        'view-distance': '4',
        'simulation-distance': '4'
      }
    })
    server = startVibeCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, host, 10_000)

    const cases = [
      {
        name: 'play chat command cannot alias selected-known-packs',
        username: uniqueUsername('Cmd'),
        setup: enterConfiguration,
        packetId: 7,
        payload: chatCommandPayload('say should-not-run')
      },
      {
        name: 'play chat message is rejected during configuration',
        username: uniqueUsername('Chat'),
        setup: enterConfiguration,
        packetId: 9,
        payload: chatPayload('should-not-run')
      },
      {
        name: 'play command suggestion is rejected during configuration',
        username: uniqueUsername('Suggest'),
        setup: enterConfiguration,
        packetId: 15,
        payload: commandSuggestionPayload('/list')
      },
      {
        name: 'play movement is rejected during configuration',
        username: uniqueUsername('Move'),
        setup: enterConfiguration,
        packetId: 31,
        payload: movePlayerPosRotPayload()
      },
      {
        name: 'play inventory click is rejected during configuration',
        username: uniqueUsername('Inv'),
        setup: enterConfiguration,
        packetId: 18,
        payload: containerClickPayload()
      },
      {
        name: 'early finish-configuration cannot enter play before server finish',
        username: uniqueUsername('Fin'),
        setup: enterConfiguration,
        packetId: 3,
        payload: Buffer.alloc(0)
      }
    ]

    for (const scenario of cases) {
      const observed = await runRejectedPrePlayScenario(scenario)
      assertPrePlayRejected(scenario, observed)
    }
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

async function enterConfiguration (socket, reader, username) {
  socket.write(frame(0, handshakePayload(2)))
  socket.write(frame(0, writeString(username), randomUuidBytes()))
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
}

async function runRejectedPrePlayScenario (scenario) {
  const socket = await connect()
  const reader = new FrameReader(socket)
  try {
    await scenario.setup(socket, reader, scenario.username)
    socket.write(encodeClientPacket(reader.compressionThreshold, scenario.packetId, scenario.payload))
    return await nextEvent(socket, reader, 2_000)
  } finally {
    socket.destroy()
  }
}

function assertPrePlayRejected (scenario, observed) {
  assert.notEqual(observed.packet?.packetId, 49, `${scenario.name} reached play login`)
  assert.notEqual(observed.packet?.packetId, 11, `${scenario.name} reached play chunk-batch finish`)
  assert.ok(
    ['close', 'reset', 'timeout'].includes(observed.kind),
    `${scenario.name} should close, reset, or stop progressing after rejection; got ${JSON.stringify(observed)}`
  )
}

async function waitForPacket (reader, packetId) {
  const deadline = Date.now() + 10_000
  while (Date.now() < deadline) {
    const packet = await reader.nextFrame()
    if (packet.packetId === packetId) return packet
  }
  throw new Error(`timed out waiting for packet ${packetId}`)
}

async function nextEvent (socket, reader, timeoutMs) {
  return await Promise.race([
    reader.nextFrame().then(packet => ({ kind: 'packet', packet })).catch(error => ({ kind: 'close', error })),
    once(socket, 'close').then(() => ({ kind: 'close' })),
    once(socket, 'error').then(error => ({ kind: 'reset', error })),
    delay(timeoutMs).then(() => ({ kind: 'timeout' }))
  ])
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
  const socket = createConnection({ host, port })
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

function writeShort (value) {
  const payload = Buffer.alloc(2)
  payload.writeInt16BE(value)
  return payload
}

function writeByte (value) {
  const payload = Buffer.alloc(1)
  payload.writeInt8(value)
  return payload
}

function writeLong (value) {
  const payload = Buffer.alloc(8)
  payload.writeBigInt64BE(BigInt(value))
  return payload
}

function writeDouble (value) {
  const payload = Buffer.alloc(8)
  payload.writeDoubleBE(value)
  return payload
}

function writeFloat (value) {
  const payload = Buffer.alloc(4)
  payload.writeFloatBE(value)
  return payload
}

function commandSuggestionPayload (command) {
  return Buffer.concat([writeVarInt(1), writeString(command)])
}

function chatCommandPayload (command) {
  return writeString(command)
}

function chatPayload (message) {
  return Buffer.concat([
    writeString(message),
    writeLong(0),
    writeLong(0),
    Buffer.from([0]), // no signature
    writeVarInt(0), // last-seen offset
    Buffer.from([0, 0, 0]), // acknowledged bitset
    Buffer.from([0]) // checksum
  ])
}

function movePlayerPosRotPayload () {
  return Buffer.concat([
    writeDouble(0.5),
    writeDouble(112),
    writeDouble(0.5),
    writeFloat(0),
    writeFloat(0),
    Buffer.from([1])
  ])
}

function containerClickPayload () {
  return Buffer.concat([
    writeVarInt(0), // container id
    writeVarInt(0), // state id
    writeShort(0), // slot
    writeByte(0), // button
    writeVarInt(0), // pickup
    writeVarInt(0), // changed slots
    Buffer.from([0]) // empty carried HashedStack
  ])
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

async function reservePort () {
  const server = createServer()
  await new Promise((resolve, reject) => {
    server.once('error', reject)
    server.listen(0, host, resolve)
  })
  const { port } = server.address()
  await new Promise(resolve => server.close(resolve))
  return port
}

function uniqueUsername (prefix) {
  return `Sm${prefix}${crypto.randomUUID().replaceAll('-', '').slice(0, 8)}`.slice(0, 16)
}
