import assert from 'node:assert/strict'
import crypto from 'node:crypto'
import net from 'node:net'
import test from 'node:test'
import { inflateSync } from 'node:zlib'

const host = process.env.VIBECRAFT_HOST ?? '127.0.0.1'
const port = Number(process.env.VIBECRAFT_PORT ?? 25565)
const protocolVersion = Number(process.env.VIBECRAFT_PROTOCOL_VERSION ?? 775)

test('raw 26.1.2 malformed client packets are rejected at status, login, configuration, and play boundaries', { timeout: 45_000 }, async () => {
  const cases = [
    {
      name: 'status',
      run: async () => {
        const socket = await connect()
        const reader = new FrameReader(socket)
        socket.write(frame(0, handshakePayload(1)))
        socket.write(frame(99))
        return observeRejection(socket, reader, { allowedDisconnectIds: [0] })
      }
    },
    {
      name: 'login',
      run: async () => {
        const socket = await connect()
        const reader = new FrameReader(socket)
        socket.write(frame(0, handshakePayload(2)))
        socket.write(frame(2, writeVarInt(1), Buffer.from([0])))
        return observeRejection(socket, reader, { allowedDisconnectIds: [0] })
      }
    },
    {
      name: 'configuration',
      run: async () => {
        const socket = await connect()
        const reader = new FrameReader(socket)
        await enterConfiguration(socket, reader, 'BadConfig')
        socket.write(encodeClientPacket(reader.compressionThreshold, 99))
        return observeRejection(socket, reader, { allowedDisconnectIds: [2] })
      }
    },
    {
      name: 'play',
      run: async () => {
        const socket = await connect()
        const reader = new FrameReader(socket)
        await enterPlay(socket, reader, 'BadPlay')
        socket.write(encodeClientPacket(reader.compressionThreshold, 999))
        return observeRejection(socket, reader, { allowedDisconnectIds: [32] })
      }
    }
  ]

  for (const malformed of cases) {
    const result = await malformed.run()
    assert.equal(result.rejected, true, `${malformed.name} malformed packet should be rejected`)
    assert.ok(['disconnect', 'close', 'reset'].includes(result.kind), `${malformed.name} rejection should close or disconnect`)
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
  while (true) {
    const packet = await reader.nextFrame()
    if (packet.packetId === 14) break
  }
}

async function enterPlay (socket, reader, username) {
  await enterConfiguration(socket, reader, username)
  socket.write(encodeClientPacket(reader.compressionThreshold, 7, writeVarInt(0)))
  while (true) {
    const packet = await reader.nextFrame()
    if (packet.packetId === 3) {
      socket.write(encodeClientPacket(reader.compressionThreshold, 3))
      break
    }
  }

  const login = await reader.nextFrame()
  assert.equal(login.packetId, 49)
  while (true) {
    const packet = await reader.nextFrame()
    if (packet.packetId === 11) break
  }
}

async function observeRejection (socket, reader, { allowedDisconnectIds }) {
  try {
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
          socket.destroy()
          return {
            rejected: true,
            kind: 'disconnect',
            packetId: observed.packet.packetId
          }
        }
        continue
      }

      socket.destroy()
      return {
        rejected: observed.kind === 'close' || observed.kind === 'reset',
        kind: observed.kind
      }
    }

    socket.destroy()
    return {
      rejected: false,
      kind: 'timeout'
    }
  } finally {
    socket.destroy()
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
