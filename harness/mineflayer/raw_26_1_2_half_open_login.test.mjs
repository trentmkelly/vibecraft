import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import crypto from 'node:crypto'
import net from 'node:net'
import test from 'node:test'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)
const host = process.env.RUSTCRAFT_HOST ?? '127.0.0.1'
const port = Number(process.env.RUSTCRAFT_PORT ?? 25565)
const protocolVersion = Number(process.env.RUSTCRAFT_PROTOCOL_VERSION ?? 775)

test('raw 26.1.2 half-open login sockets time out and leave later login usable', { timeout: 150_000 }, async () => {
  const phases = [
    ['tcp_connect', openIdleTcpConnect],
    ['handshake', openIdleAfterHandshake],
    ['login_start', openIdleAfterLoginStart]
  ]

  for (const [phase, open] of phases) {
    const username = `Half${phase.replaceAll('_', '').slice(0, 10)}`
    const idle = await open(username)
    assert.equal(idle.closed, true, `${phase} socket should be closed by server timeout`)
    assert.ok(idle.elapsedMs >= 25_000, `${phase} timeout should not be an immediate refusal`)
    assert.ok(idle.elapsedMs < 45_000, `${phase} timeout should stay near the configured 30s read timeout`)

    const retry = await runJoinProbe(username)
    assert.equal(retry.ok, true, `${phase} retry should reach play after half-open cleanup`)
    assert.equal(retry.joinState.profile.name, username)
    assert.ok(retry.play.some(packet => packet.id === 49), `${phase} retry should receive join game`)
  }
})

async function openIdleTcpConnect () {
  return await waitForServerClose(await connect())
}

async function openIdleAfterHandshake () {
  const socket = await connect()
  socket.write(frame(0, handshakePayload()))
  return await waitForServerClose(socket)
}

async function openIdleAfterLoginStart (username) {
  const socket = await connect()
  socket.write(frame(0, handshakePayload()))
  socket.write(frame(0, writeString(username), randomUuidBytes()))
  const login = await readOnePacket(socket)
  assert.equal(login.id, 2)
  return await waitForServerClose(socket)
}

async function waitForServerClose (socket) {
  const started = Date.now()
  return await new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      socket.destroy()
      reject(new Error('half-open socket was not closed by server timeout'))
    }, 45_000)
    socket.once('close', () => {
      clearTimeout(timeout)
      resolve({ closed: true, elapsedMs: Date.now() - started })
    })
    socket.once('error', error => {
      clearTimeout(timeout)
      reject(error)
    })
  })
}

async function runJoinProbe (username) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: new URL('.', import.meta.url),
      env: {
        ...process.env,
        RUSTCRAFT_USERNAME: username
      },
      timeout: 30_000,
      maxBuffer: 1024 * 1024
    }
  )

  return JSON.parse(stdout)
}

async function connect () {
  const socket = net.createConnection({ host, port })
  await new Promise((resolve, reject) => {
    socket.once('connect', resolve)
    socket.once('error', reject)
  })
  return socket
}

function handshakePayload () {
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

function writeString (value) {
  const data = Buffer.from(value, 'utf8')
  return Buffer.concat([writeVarInt(data.length), data])
}

async function readOnePacket (socket) {
  let buffer = Buffer.alloc(0)
  return await new Promise((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error('timed out waiting for packet')), 5000)
    socket.on('data', onData)
    socket.once('error', onError)

    function onData (chunk) {
      buffer = Buffer.concat([buffer, chunk])
      const length = readVarInt(buffer)
      if (!length) return
      const end = length.offset + length.value
      if (buffer.length < end) return
      cleanup()
      const payload = buffer.subarray(length.offset, end)
      const packetId = readVarInt(payload)
      resolve({ id: packetId.value, body: payload.subarray(packetId.offset) })
    }

    function onError (error) {
      cleanup()
      reject(error)
    }

    function cleanup () {
      clearTimeout(timeout)
      socket.off('data', onData)
      socket.off('error', onError)
    }
  })
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

function randomUuidBytes () {
  return Buffer.from(crypto.randomUUID().replaceAll('-', ''), 'hex')
}
