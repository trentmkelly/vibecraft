import assert from 'node:assert/strict'
import crypto from 'node:crypto'
import net from 'node:net'
import { rm } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { inflateSync } from 'node:zlib'

import {
  createTempWorld,
  startRustCraft,
  stopServer,
  waitForPort,
  writeOfflineServerFiles
} from './runner.mjs'

const here = new URL('.', import.meta.url)
const repoRoot = path.resolve(here.pathname, '..', '..')
const binary = path.join(repoRoot, 'target', 'debug', 'rustcraft')
const host = '127.0.0.1'
const protocolVersion = Number(process.env.RUSTCRAFT_PROTOCOL_VERSION ?? 775)

test('raw 26.1.2 rejects chat and command packets before play readiness', { timeout: 60_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('rustcraft-command-before-ready-')
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
    server = startRustCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, host, 10_000)

    const cases = [
      ['login-command-suggestion', socket => {
        socket.write(frame(0, handshakePayload(port, 2)))
        socket.write(frame(15, commandSuggestionPayload('/list')))
      }, [0]],
      ['login-chat', socket => {
        socket.write(frame(0, handshakePayload(port, 2)))
        socket.write(frame(9, chatPayload('too early')))
      }, [0]],
      ['configuration-command-suggestion', async (socket, reader) => {
        await enterConfiguration(socket, reader, port, 'CmdCfg')
        socket.write(encodeClientPacket(reader.compressionThreshold, 15, commandSuggestionPayload('/list')))
      }, [2]]
    ]

    for (const [name, sendEarlyPacket, allowedDisconnectIds] of cases) {
      const socket = await connect(port)
      const reader = new FrameReader(socket)
      try {
        await sendEarlyPacket(socket, reader)
        const result = await observeRejection(socket, reader, { allowedDisconnectIds })
        assert.equal(result.rejected, true, `${name} should be rejected before play readiness`)
      } finally {
        socket.destroy()
      }
    }
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

async function enterConfiguration (socket, reader, port, username) {
  socket.write(frame(0, handshakePayload(port, 2)))
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
  while (true) {
    const packet = await reader.nextFrame()
    if (packet.packetId === 14) return
  }
}

async function observeRejection (socket, reader, { allowedDisconnectIds }) {
  const deadline = Date.now() + 5_000
  while (Date.now() < deadline) {
    const observed = await Promise.race([
      reader.nextFrame()
        .then(packet => ({ kind: 'packet', packet }))
        .catch(error => ({ kind: 'close', error })),
      once(socket, 'close').then(() => ({ kind: 'close' })),
      once(socket, 'error').then(error => ({ kind: 'reset', error })),
      delay(Math.max(1, deadline - Date.now())).then(() => ({ kind: 'timeout' }))
    ])

    if (observed.kind === 'packet') {
      if (allowedDisconnectIds.includes(observed.packet.packetId)) {
        return {
          rejected: true,
          kind: 'disconnect',
          packetId: observed.packet.packetId
        }
      }
      continue
    }

    return {
      rejected: observed.kind === 'close' || observed.kind === 'reset',
      kind: observed.kind
    }
  }

  return {
    rejected: false,
    kind: 'timeout'
  }
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

async function connect (port) {
  const socket = net.createConnection({ host, port })
  socket.setMaxListeners(64)
  await Promise.race([
    once(socket, 'connect'),
    once(socket, 'error').then(error => Promise.reject(error))
  ])
  return socket
}

function handshakePayload (port, nextState) {
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

function commandSuggestionPayload (command) {
  return Buffer.concat([writeVarInt(1), writeString(command)])
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

function delay (ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

async function reservePort () {
  const server = net.createServer()
  await new Promise((resolve, reject) => {
    server.once('error', reject)
    server.listen(0, host, resolve)
  })
  const { port } = server.address()
  await new Promise(resolve => server.close(resolve))
  return port
}
