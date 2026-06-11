import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import crypto from 'node:crypto'
import { rm } from 'node:fs/promises'
import net from 'node:net'
import path from 'node:path'
import test from 'node:test'
import { promisify } from 'node:util'

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

test('raw 26.1.2 socket cleanup survives aborts during handshake, login, configuration, and play entry', { timeout: 180_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-socket-cleanup-')
  let server
  const phases = [
    ['tcp_connect', abortTcpConnect],
    ['handshake', abortAfterHandshake],
    ['login_start', abortAfterLoginStart],
    ['compression', abortAfterCompression],
    ['login_success', abortJoinProbe],
    ['configuration', abortJoinProbe],
    ['play_entry', abortJoinProbe]
  ]

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

    for (const [phase, abort] of phases) {
      const username = `Cl${phase.replaceAll('_', '').slice(0, 8)}${crypto.randomUUID().replaceAll('-', '').slice(0, 5)}`
      const aborted = await abort(port, phase, username)
      assert.equal(aborted.ok, true, `${phase} abort should complete locally`)

      const retry = await runJoinProbe(port, username, { VIBECRAFT_RAW_PROBE_KEEPALIVE_MS: '17000' })
      assert.equal(retry.ok, true, `${phase} retry should reach play`)
      assert.equal(retry.joinState.profile.name, username)
      assert.ok(retry.configPacketCount > 0, `${phase} retry should receive config packets`)
      assert.ok(retry.playPacketCount > 0, `${phase} retry should receive play packets`)
      assert.ok(retry.keepAliveReplies >= 1, `${phase} retry should prove no stale keepalive task blocks a fresh session`)
    }
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

async function abortTcpConnect (port) {
  const socket = await connect(port)
  socket.destroy()
  return { ok: true }
}

async function abortAfterHandshake (port) {
  const socket = await connect(port)
  socket.write(frame(0, handshakePayload(port)))
  socket.destroy()
  return { ok: true }
}

async function abortAfterLoginStart (port, _phase, username) {
  const socket = await connect(port)
  socket.write(frame(0, handshakePayload(port)))
  socket.write(frame(0, writeString(username), randomUuidBytes()))
  socket.destroy()
  return { ok: true }
}

async function abortAfterCompression (port, _phase, username) {
  const socket = await connect(port)
  socket.write(frame(0, handshakePayload(port)))
  socket.write(frame(0, writeString(username), randomUuidBytes()))
  const packet = await readUncompressedFrame(socket)
  assert.equal(packet.id, 3, 'expected login compression before abort')
  socket.destroy()
  return { ok: true }
}

async function abortJoinProbe (port, phase, username) {
  const abortAfter = {
    login_success: 'login_success',
    configuration: 'registry_sync',
    play_entry: 'first_chunk'
  }[phase]
  const result = await runJoinProbe(port, username, { VIBECRAFT_RAW_PROBE_ABORT_AFTER: abortAfter })
  assert.equal(result.aborted, true)
  assert.equal(result.phase, abortAfter)
  return { ok: true }
}

async function runJoinProbe (port, username, env = {}) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: here,
      env: {
        ...process.env,
        VIBECRAFT_HOST: host,
        VIBECRAFT_PORT: String(port),
        VIBECRAFT_USERNAME: username,
        VIBECRAFT_RAW_PROBE_OUTPUT: 'summary',
        ...env
      },
      timeout: 30_000,
      maxBuffer: 1024 * 1024
    }
  )

  return JSON.parse(stdout)
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

async function readUncompressedFrame (socket) {
  let buffer = Buffer.alloc(0)
  while (true) {
    const decoded = tryReadFrame(buffer)
    if (decoded) return decoded
    const chunk = await new Promise((resolve, reject) => {
      socket.once('data', resolve)
      socket.once('error', reject)
      socket.once('close', () => reject(new Error('socket closed before frame')))
    })
    buffer = Buffer.concat([buffer, chunk])
  }
}

function tryReadFrame (buffer) {
  const length = readVarInt(buffer)
  if (!length) return null
  const end = length.offset + length.value
  if (buffer.length < end) return null
  const payload = buffer.subarray(length.offset, end)
  const id = readVarInt(payload)
  if (!id) return null
  return {
    id: id.value,
    body: payload.subarray(id.offset)
  }
}

function frame (packetId, ...parts) {
  const payload = Buffer.concat([writeVarInt(packetId), ...parts])
  return Buffer.concat([writeVarInt(payload.length), payload])
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
  const server = net.createServer()
  await new Promise((resolve, reject) => {
    server.once('error', reject)
    server.listen(0, host, resolve)
  })
  const { port } = server.address()
  await new Promise(resolve => server.close(resolve))
  return port
}
