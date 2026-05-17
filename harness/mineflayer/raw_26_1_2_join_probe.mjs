import net from 'node:net'
import crypto from 'node:crypto'

const host = process.env.RUSTCRAFT_HOST ?? '127.0.0.1'
const port = Number(process.env.RUSTCRAFT_PORT ?? 25565)
const username = process.env.RUSTCRAFT_USERNAME ?? 'RustCraftProbe'
const protocolVersion = 775

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
    config.push({ id: packet.id, length: packet.length })
    if (packet.id === 3) break
  }
  const configIds = config.map(packet => packet.id)
  for (const id of [12, 7, 13, 3]) {
    if (!configIds.includes(id)) throw new Error(`missing configuration packet ${id}`)
  }
  socket.write(frame(3))

  const play = []
  for (let i = 0; i < 3; i++) {
    const packet = await reader.nextPacket()
    play.push({ id: packet.id, length: packet.length })
  }
  for (const id of [49, 105, 72]) {
    if (!play.some(packet => packet.id === id)) throw new Error(`missing play packet ${id}`)
  }

  socket.end()
  console.log(JSON.stringify({ ok: true, host, port, login: login.id, config, play }, null, 2))
}

main().catch(error => {
  console.error(JSON.stringify({ ok: false, error: error.message, stack: error.stack }, null, 2))
  process.exit(1)
})
