import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import crypto from 'node:crypto'
import { createServer } from 'node:net'
import { rm } from 'node:fs/promises'
import net from 'node:net'
import path from 'node:path'
import test from 'node:test'
import { promisify } from 'node:util'
import { inflateSync } from 'node:zlib'

import {
  createTempWorld,
  startVibeCraft,
  stopServer,
  waitForPort,
  writeOfflineServerFiles
} from './runner.mjs'

const execFileAsync = promisify(execFile)
const here = new URL('.', import.meta.url)
const repoRoot = path.resolve(here.pathname, '..', '..')
const binary = path.join(repoRoot, 'target', 'debug', 'vibecraft')
const host = '127.0.0.1'
const protocolVersion = Number(process.env.VIBECRAFT_PROTOCOL_VERSION ?? 775)
const clientboundKeepAlivePacketId = 44
const serverboundKeepAlivePacketId = 28

test('raw 26.1.2 play probe survives multiple keepalive intervals and rejects duplicate replies', { timeout: 90_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-play-keepalive-')
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

    const result = await runJoinProbe(port)
    assert.equal(result.ok, true)
    assert.ok(result.keepAliveReplies >= 2, 'expected at least two keepalive round trips')
    assert.ok(result.play.some(packet => packet.id === clientboundKeepAlivePacketId), 'expected clientbound keep_alive packet')

    const duplicate = await duplicateKeepAliveProbe(port)
    assert.equal(duplicate.firstKeepAliveSeen, true)
    assert.equal(duplicate.closedAfterDuplicate, true)
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

async function runJoinProbe (port) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: here,
      env: {
        ...process.env,
        VIBECRAFT_HOST: host,
        VIBECRAFT_PORT: String(port),
        VIBECRAFT_RAW_PROBE_KEEPALIVE_MS: '32000'
      },
      timeout: 45_000,
      maxBuffer: 1024 * 1024
    }
  )

  return JSON.parse(stdout)
}

async function duplicateKeepAliveProbe (port) {
  const socket = await connect(port)
  const reader = new FrameReader(socket)
  try {
    await completeJoin(socket, reader, port, `KeepDup${crypto.randomUUID().replaceAll('-', '').slice(0, 6)}`)

    const keepAlive = await waitForPacket(reader, clientboundKeepAlivePacketId, 20_000)
    socket.write(encodeClientPacket(reader.compressionThreshold, serverboundKeepAlivePacketId, keepAlive.body))
    socket.write(encodeClientPacket(reader.compressionThreshold, serverboundKeepAlivePacketId, keepAlive.body))

    const closedAfterDuplicate = await waitForClose(socket, 10_000)
    return { firstKeepAliveSeen: true, closedAfterDuplicate }
  } finally {
    socket.destroy()
  }
}

async function completeJoin (socket, reader, port, username) {
  socket.write(frame(0, handshakePayload(port)))
  socket.write(frame(0, writeString(username), randomUuidBytes()))

  let login = await reader.nextPacket()
  if (login.id === 3) {
    const threshold = readVarInt(login.body, 0)
    reader.setCompression(threshold.value)
    login = await reader.nextPacket()
  }
  assert.equal(login.id, 2)
  socket.write(encodeClientPacket(reader.compressionThreshold, 3))

  while (true) {
    const packet = await reader.nextPacket()
    if (packet.id === 14) {
      socket.write(encodeClientPacket(reader.compressionThreshold, 7, writeVarInt(0)))
    }
    if (packet.id === 3) {
      socket.write(encodeClientPacket(reader.compressionThreshold, 3))
      break
    }
  }

  while (true) {
    const packet = await reader.nextPacket()
    if (packet.id === 49) break
  }
}

async function waitForPacket (reader, packetId, timeoutMs) {
  const deadline = Date.now() + timeoutMs
  while (Date.now() < deadline) {
    const packet = await Promise.race([
      reader.nextPacket(),
      new Promise(resolve => setTimeout(() => resolve(null), Math.max(1, deadline - Date.now())))
    ])
    if (!packet) break
    if (packet.id === packetId) return packet
  }
  throw new Error(`packet ${packetId} not observed within ${timeoutMs}ms`)
}

async function waitForClose (socket, timeoutMs) {
  return await new Promise(resolve => {
    if (socket.destroyed) return resolve(true)
    const timeout = setTimeout(() => resolve(false), timeoutMs)
    socket.once('close', () => {
      clearTimeout(timeout)
      resolve(true)
    })
  })
}

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

  nextPacket () {
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
      this.waiters.shift().resolve(decoded.packet)
    }
  }

  setCompression (threshold) {
    this.compressionThreshold = threshold
  }

  rejectAll (error) {
    for (const waiter of this.waiters.splice(0)) waiter.reject(error)
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
    packet: {
      id: packetId.value,
      body: payload.subarray(packetId.offset)
    }
  }
}

async function connect (port) {
  const socket = net.createConnection({ host, port })
  await new Promise((resolve, reject) => {
    socket.once('connect', resolve)
    socket.once('error', reject)
  })
  return socket
}

function handshakePayload (port) {
  return Buffer.concat([
    writeVarInt(protocolVersion),
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
