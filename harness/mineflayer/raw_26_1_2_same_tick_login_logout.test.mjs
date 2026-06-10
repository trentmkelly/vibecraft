import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import crypto from 'node:crypto'
import net from 'node:net'
import { createServer } from 'node:net'
import { rm } from 'node:fs/promises'
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

test('raw 26.1.2 same-tick login/logout cleanup permits immediate same-name rejoins', { timeout: 75_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-same-tick-login-logout-')
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

    const aborts = [
      abortAfterHandshake(port, 'SameTickA'),
      abortAfterLoginStart(port, 'SameTickB'),
      abortJoinProbe(port, 'SameTickC', 'login_success'),
      abortJoinProbe(port, 'SameTickD', 'registry_sync'),
      abortJoinProbe(port, 'SameTickE', 'first_chunk')
    ]
    const aborted = await Promise.all(aborts)
    assert.deepEqual(aborted.map(result => result.ok), aborted.map(() => true))

    const retries = await Promise.all(aborted.map(result => runJoinProbe(port, result.username)))
    for (const [index, retry] of retries.entries()) {
      const username = aborted[index].username
      assert.equal(retry.ok, true)
      assert.equal(retry.joinState.profile.name, username)
      assert.ok(retry.config.some(packet => packet.id === 3), `${username} should finish configuration after abort cleanup`)
      assert.ok(retry.play.some(packet => packet.id === 49), `${username} should reach play after abort cleanup`)
      assert.ok(retry.play.some(packet => packet.id === 70), `${username} should receive tab-list profile after abort cleanup`)
      assert.equal(retry.joinState.lastReceivedChunk, 8)
    }
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

async function abortAfterHandshake (port, username) {
  const socket = await connect(port)
  socket.write(frame(0, handshakePayload(port)))
  socket.destroy()
  return { ok: true, username }
}

async function abortAfterLoginStart (port, username) {
  const socket = await connect(port)
  socket.write(frame(0, handshakePayload(port)))
  socket.write(frame(0, writeString(username), randomUuidBytes()))
  socket.destroy()
  return { ok: true, username }
}

async function abortJoinProbe (port, username, abortAfter) {
  const result = await runJoinProbe(port, username, { VIBECRAFT_RAW_PROBE_ABORT_AFTER: abortAfter })
  assert.equal(result.ok, true)
  assert.equal(result.aborted, true)
  assert.equal(result.phase, abortAfter)
  return { ok: true, username }
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
