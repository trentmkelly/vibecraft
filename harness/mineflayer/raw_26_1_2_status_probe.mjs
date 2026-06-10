import { createConnection } from 'node:net'

const host = process.env.VIBECRAFT_HOST ?? '127.0.0.1'
const port = Number(process.env.VIBECRAFT_PORT ?? 25565)
const timeoutMs = Number(process.env.VIBECRAFT_TIMEOUT_MS ?? 5000)
const targetProtocolVersion = 775
const protocolVersion = Number(process.env.VIBECRAFT_PROTOCOL_VERSION ?? 775)
const versionName = '26.1.2'

function writeVarInt (value) {
  let remaining = value >>> 0
  const bytes = []
  do {
    let temp = remaining & 0x7f
    remaining >>>= 7
    if (remaining !== 0) temp |= 0x80
    bytes.push(temp)
  } while (remaining !== 0)
  return Buffer.from(bytes)
}

function readVarInt (buffer, offset = 0) {
  let value = 0
  let position = 0
  let currentOffset = offset

  while (currentOffset < buffer.length) {
    const current = buffer[currentOffset++]
    value |= (current & 0x7f) << (7 * position)
    if ((current & 0x80) === 0) {
      return { value, size: currentOffset - offset }
    }
    position++
    if (position > 4) throw new Error('VarInt too large')
  }

  return null
}

function writeString (value) {
  const bytes = Buffer.from(value, 'utf8')
  return Buffer.concat([writeVarInt(bytes.length), bytes])
}

function readString (buffer, offset) {
  const length = readVarInt(buffer, offset)
  if (!length) throw new Error('incomplete string length')
  const start = offset + length.size
  const end = start + length.value
  if (end > buffer.length) throw new Error('incomplete string payload')
  return {
    value: buffer.subarray(start, end).toString('utf8'),
    size: length.size + length.value
  }
}

function frame (packetId, payload = Buffer.alloc(0)) {
  const body = Buffer.concat([writeVarInt(packetId), payload])
  return Buffer.concat([writeVarInt(body.length), body])
}

function handshakePacket () {
  const payload = Buffer.concat([
    writeVarInt(protocolVersion),
    writeString(host),
    Buffer.from([(port >> 8) & 0xff, port & 0xff]),
    writeVarInt(1)
  ])
  return frame(0, payload)
}

function pingPayload (value) {
  const payload = Buffer.alloc(8)
  payload.writeBigInt64BE(BigInt(value))
  return payload
}

function parsePacket (buffer) {
  const length = readVarInt(buffer, 0)
  if (!length) return null
  const packetStart = length.size
  const packetEnd = packetStart + length.value
  if (buffer.length < packetEnd) return null

  const id = readVarInt(buffer, packetStart)
  if (!id) throw new Error('incomplete packet id')
  const bodyStart = packetStart + id.size
  return {
    packet: {
      id: id.value,
      body: buffer.subarray(bodyStart, packetEnd)
    },
    rest: buffer.subarray(packetEnd)
  }
}

async function runStatusProbe () {
  const socket = createConnection({ host, port })
  socket.setTimeout(timeoutMs)

  let buffer = Buffer.alloc(0)
  const packets = []
  const pongPayload = 0x0102030405060708n

  socket.on('data', chunk => {
    buffer = Buffer.concat([buffer, chunk])
    while (true) {
      const parsed = parsePacket(buffer)
      if (!parsed) break
      packets.push(parsed.packet)
      buffer = parsed.rest
    }
  })

  await onceSocket(socket, 'connect')
  socket.write(handshakePacket())
  socket.write(frame(0))
  const statusPacket = await waitForPacket(packets, packet => packet.id === 0)
  const statusText = readString(statusPacket.body, 0).value
  const status = JSON.parse(statusText)

  socket.write(frame(1, pingPayload(pongPayload)))
  const pongPacket = await waitForPacket(packets, packet => packet.id === 1)
  const pong = pongPacket.body.readBigInt64BE(0)
  socket.end()

  return {
    ok: true,
    status,
    pong: pong.toString(),
    expectedPong: pongPayload.toString(),
    assertions: {
      protocol: status.version?.protocol === targetProtocolVersion,
      versionName: status.version?.name === versionName,
      motdText: typeof status.description?.text === 'string',
      playerCounts: Number.isInteger(status.players?.online) && Number.isInteger(status.players?.max),
      pongEcho: pong === pongPayload
    }
  }
}

function onceSocket (socket, event) {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error(`timed out waiting for ${event}`)), timeoutMs)
    socket.once(event, value => {
      clearTimeout(timeout)
      resolve(value)
    })
    socket.once('error', error => {
      clearTimeout(timeout)
      reject(error)
    })
    socket.once('timeout', () => {
      clearTimeout(timeout)
      reject(new Error('socket timed out'))
    })
  })
}

function waitForPacket (packets, predicate) {
  const existing = packets.find(predicate)
  if (existing) return Promise.resolve(existing)

  return new Promise((resolve, reject) => {
    const start = Date.now()
    const interval = setInterval(() => {
      const packet = packets.find(predicate)
      if (packet) {
        clearInterval(interval)
        resolve(packet)
      } else if (Date.now() - start > timeoutMs) {
        clearInterval(interval)
        reject(new Error('timed out waiting for status packet'))
      }
    }, 10)
  })
}

try {
  const result = await runStatusProbe()
  if (!Object.values(result.assertions).every(Boolean)) {
    result.ok = false
    process.exitCode = 1
  }
  console.log(JSON.stringify(result, null, 2))
} catch (error) {
  console.error(JSON.stringify({
    ok: false,
    error: error.message,
    stack: error.stack
  }, null, 2))
  process.exitCode = 1
}
