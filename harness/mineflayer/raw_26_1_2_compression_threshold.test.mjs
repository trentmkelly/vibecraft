import assert from 'node:assert/strict'
import crypto from 'node:crypto'
import { createServer } from 'node:net'
import { rm } from 'node:fs/promises'
import net from 'node:net'
import path from 'node:path'
import { inflateSync } from 'node:zlib'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import {
  createTempWorld,
  startVibeCraft,
  stopServer,
  waitForPort,
  writeOfflineServerFiles
} from './runner.mjs'

const here = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(here, '..', '..')
const binary = path.join(repoRoot, 'target', 'debug', 'vibecraft')
const host = '127.0.0.1'
const protocolVersion = Number(process.env.VIBECRAFT_PROTOCOL_VERSION ?? 775)

test('raw 26.1.2 compression thresholds reach play and decode large registry packets', { timeout: 90_000 }, async () => {
  const cases = [
    { threshold: -1, compression: false },
    { threshold: 32, compression: true, expectCompressedRegistry: true },
    { threshold: 256, compression: true, expectCompressedRegistry: true }
  ]

  for (const entry of cases) {
    const port = await reservePort()
    const root = await createTempWorld(`vibecraft-compression-${entry.threshold}-`)
    let server
    try {
      await writeOfflineServerFiles(root, {
        port,
        levelName: 'world',
        properties: { 'network-compression-threshold': String(entry.threshold) }
      })
      server = startVibeCraft({ binary, root, port, levelName: 'world' })
      await waitForPort(port, host, 10_000)

      const result = await rawJoin(port, `Zip${entry.threshold < 0 ? 'Off' : entry.threshold}`)
      assert.equal(result.loginSuccess, true, `threshold ${entry.threshold} should complete login`)
      assert.equal(result.compressionThreshold, entry.compression ? entry.threshold : null)
      assert.equal(result.playLogin, true, `threshold ${entry.threshold} should reach play`)
      assert.ok(result.largeRegistryDecoded, `threshold ${entry.threshold} should decode a large registry packet`)
      if (entry.expectCompressedRegistry) {
        assert.ok(result.compressedPackets > 0, `threshold ${entry.threshold} should compress at least one large packet`)
      } else {
        assert.equal(result.compressedPackets, 0)
      }
    } finally {
      if (server) await stopServer(server.child)
      await rm(root, { recursive: true, force: true })
    }
  }
})

async function rawJoin (port, username) {
  const socket = await connect(port)
  const reader = new FrameReader(socket)
  let compressionThreshold = null
  let compressedPackets = 0
  let largeRegistryDecoded = false
  try {
    socket.write(frame(0, handshakePayload(port, 2)))
    socket.write(frame(0, writeString(username), randomUuidBytes()))

    let loginSuccess = false
    while (!loginSuccess) {
      const packet = await reader.nextPacket(compressionThreshold)
      if (packet.packetId === 3) {
        compressionThreshold = readVarInt(packet.body, 0).value
      } else if (packet.packetId === 2) {
        loginSuccess = true
      } else {
        throw new Error(`unexpected login packet ${packet.packetId}`)
      }
    }

    socket.write(encodeClientPacket(compressionThreshold, 3))

    while (true) {
      const packet = await reader.nextPacket(compressionThreshold)
      compressedPackets += packet.compressed ? 1 : 0
      if (packet.packetId === 7 && packet.body.length > 512) largeRegistryDecoded = true
      if (packet.packetId === 14) socket.write(encodeClientPacket(compressionThreshold, 7, writeVarInt(0)))
      if (packet.packetId === 3) {
        socket.write(encodeClientPacket(compressionThreshold, 3))
        break
      }
    }

    let playLogin = false
    while (!playLogin) {
      const packet = await reader.nextPacket(compressionThreshold)
      compressedPackets += packet.compressed ? 1 : 0
      if (packet.packetId === 49) playLogin = true
    }

    return { loginSuccess, compressionThreshold, compressedPackets, largeRegistryDecoded, playLogin }
  } finally {
    socket.destroy()
  }
}

class FrameReader {
  constructor (socket) {
    this.buffer = Buffer.alloc(0)
    this.waiters = []
    socket.on('data', chunk => {
      this.buffer = Buffer.concat([this.buffer, chunk])
      this.pump()
    })
    socket.on('error', error => this.rejectAll(error))
    socket.on('close', () => this.rejectAll(new Error('socket closed')))
  }

  nextPacket (compressionThreshold) {
    return new Promise((resolve, reject) => {
      this.waiters.push({ resolve, reject, compressionThreshold })
      this.pump()
    })
  }

  pump () {
    while (this.waiters.length > 0) {
      const waiter = this.waiters[0]
      const decoded = tryDecodeFrame(this.buffer, waiter.compressionThreshold)
      if (!decoded) return
      this.buffer = this.buffer.subarray(decoded.frameLength)
      this.waiters.shift().resolve(decoded.packet)
    }
  }

  rejectAll (error) {
    for (const waiter of this.waiters.splice(0)) waiter.reject(error)
  }
}

function tryDecodeFrame (buffer, compressionThreshold) {
  const length = readVarInt(buffer)
  if (!length) return null
  const end = length.offset + length.value
  if (buffer.length < end) return null

  let payload = buffer.subarray(length.offset, end)
  let compressed = false
  if (compressionThreshold != null) {
    const dataLength = readVarInt(payload)
    if (!dataLength) return null
    const body = payload.subarray(dataLength.offset)
    if (dataLength.value > 0) {
      payload = inflateSync(body)
      assert.equal(payload.length, dataLength.value)
      compressed = true
    } else {
      payload = body
    }
  }

  const packetId = readVarInt(payload)
  if (!packetId) throw new Error('packet missing id')
  return {
    frameLength: end,
    packet: {
      packetId: packetId.value,
      body: payload.subarray(packetId.offset),
      compressed
    }
  }
}

function encodeClientPacket (compressionThreshold, packetId, ...parts) {
  const payload = Buffer.concat([writeVarInt(packetId), ...parts])
  if (compressionThreshold == null) return rawFrame(payload)
  if (payload.length < compressionThreshold) {
    return rawFrame(Buffer.concat([writeVarInt(0), payload]))
  }
  throw new Error('test client does not send compressed serverbound payloads')
}

function frame (packetId, ...parts) {
  return rawFrame(Buffer.concat([writeVarInt(packetId), ...parts]))
}

function rawFrame (payload) {
  return Buffer.concat([writeVarInt(payload.length), payload])
}

function handshakePayload (port, nextState) {
  return Buffer.concat([
    writeVarInt(protocolVersion),
    writeString(host),
    Buffer.from([(port >> 8) & 0xff, port & 0xff]),
    writeVarInt(nextState)
  ])
}

async function connect (port) {
  const socket = net.createConnection({ host, port })
  await Promise.race([
    once(socket, 'connect'),
    once(socket, 'error').then(error => Promise.reject(error))
  ])
  return socket
}

async function reservePort () {
  const server = createServer()
  await new Promise((resolve, reject) => {
    server.once('error', reject)
    server.listen(0, host, resolve)
  })
  const { port } = server.address()
  await new Promise((resolve, reject) => {
    server.close(error => error ? reject(error) : resolve())
  })
  return port
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
