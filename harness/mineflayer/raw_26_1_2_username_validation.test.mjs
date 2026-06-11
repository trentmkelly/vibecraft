import assert from 'node:assert/strict'
import crypto from 'node:crypto'
import { rm } from 'node:fs/promises'
import net from 'node:net'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'
import { inflateSync } from 'node:zlib'

import {
  createTempWorld,
  startVibeCraft,
  stopServer,
  waitForPort,
  writeOfflineServerFiles
} from './runner.mjs'

const host = '127.0.0.1'
const protocolVersion = 775
const here = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(here, '..', '..')
const binary = path.join(repoRoot, 'target', 'debug', 'vibecraft')

test('raw 26.1.2 offline login validates usernames like Java StringUtil', { timeout: 75_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-username-validation-')
  let server

  try {
    await writeOfflineServerFiles(root, { port, levelName: 'world' })
    server = startVibeCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, host, 10_000)

    for (const name of ['Steve', 'abcdefghijklmnop', 'CaseName', 'casename', 'dash-name', 'period.name']) {
      const login = await withCase(`valid ${name}`, () => loginUntilFinished(port, name))
      assert.equal(login.name, name)
      assert.equal(login.uuid, offlineUuid(name))
    }

    const upper = await connect(port)
    const lower = await connect(port)
    try {
      const upperLogin = await withCase('duplicate casing CaseTwin', () => loginUntilFinishedOnSocket(upper, 'CaseTwin', port))
      const lowerLogin = await withCase('duplicate casing casetwin', () => loginUntilFinishedOnSocket(lower, 'casetwin', port))
      assert.notEqual(upperLogin.uuid, lowerLogin.uuid)
      assert.equal(upperLogin.name, 'CaseTwin')
      assert.equal(lowerLogin.name, 'casetwin')
    } finally {
      upper.destroy()
      lower.destroy()
    }

    for (const name of ['has space', 'newline\nname', 'nonasciié', 'seventeen_chars__']) {
      const disconnect = await withCase(`invalid ${JSON.stringify(name)}`, () => loginUntilDisconnect(port, name))
      assert.equal(disconnect.packetId, 0)
      assert.match(disconnect.reasonUtf8, /disconnect\.genericReason/)
      assert.match(disconnect.reasonUtf8, /Internal Exception:/)
    }
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

async function loginUntilFinished (port, name) {
  const socket = await connect(port)
  try {
    return await loginUntilFinishedOnSocket(socket, name, port)
  } finally {
    socket.destroy()
  }
}

async function withCase (label, action) {
  try {
    return await action()
  } catch (error) {
    error.message = `${label}: ${error.message}`
    throw error
  }
}

async function loginUntilFinishedOnSocket (socket, name, port) {
  const reader = new FrameReader(socket)
  socketLoginHello(socket, name, port)
  let packet = await reader.nextFrame()
  if (packet.packetId === 3) {
    const threshold = readVarInt(packet.body, 0)
    assert.ok(threshold, 'missing compression threshold')
    reader.setCompression(threshold.value)
    packet = await reader.nextFrame()
  }
  assert.equal(packet.packetId, 2)
  return decodeLoginFinished(packet.body)
}

async function loginUntilDisconnect (port, name) {
  const socket = await connect(port)
  try {
    const reader = new FrameReader(socket)
    socketLoginHello(socket, name, port)
    const packet = await reader.nextFrame()
    return {
      packetId: packet.packetId,
      reasonUtf8: packet.body.toString('utf8')
    }
  } finally {
    socket.destroy()
  }
}

function socketLoginHello (socket, name, port) {
  socket.write(frame(0, handshakePayload(2, port)))
  socket.write(frame(0, writeString(name), randomUuidBytes()))
}

function decodeLoginFinished (body) {
  const uuid = readUuid(body, 0)
  const name = readString(body, uuid.offset)
  return { uuid: uuid.value, name: name.value }
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

function handshakePayload (nextState, port) {
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

function readUuid (buffer, offset = 0) {
  const hex = buffer.subarray(offset, offset + 16).toString('hex')
  return {
    value: `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`,
    offset: offset + 16
  }
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

function once (emitter, event) {
  return new Promise(resolve => emitter.once(event, resolve))
}
