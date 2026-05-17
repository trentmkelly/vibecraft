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

test('raw 26.1.2 socket cleanup survives aborts during handshake, login, configuration, and play entry', { timeout: 120_000 }, async () => {
  const phases = [
    ['tcp_connect', abortTcpConnect],
    ['handshake', abortAfterHandshake],
    ['login_start', abortAfterLoginStart],
    ['login_success', abortJoinProbe],
    ['configuration', abortJoinProbe],
    ['play_entry', abortJoinProbe]
  ]

  for (const [phase, abort] of phases) {
    const username = `Clean${phase.replaceAll('_', '').slice(0, 10)}`
    const aborted = await abort(phase, username)
    assert.equal(aborted.ok, true, `${phase} abort should complete locally`)

    const retry = await runJoinProbe(username, { RUSTCRAFT_RAW_PROBE_KEEPALIVE_MS: '12000' })
    assert.equal(retry.ok, true, `${phase} retry should reach play`)
    assert.equal(retry.joinState.profile.name, username)
    assert.ok(retry.config.some(packet => packet.id === 3), `${phase} retry should finish configuration`)
    assert.ok(retry.play.some(packet => packet.id === 49), `${phase} retry should receive play login`)
    assert.ok(retry.keepAliveReplies >= 1, `${phase} retry should prove no stale keepalive task blocks a fresh session`)
  }
})

async function abortTcpConnect () {
  const socket = await connect()
  socket.destroy()
  return { ok: true }
}

async function abortAfterHandshake () {
  const socket = await connect()
  socket.write(frame(0, handshakePayload()))
  socket.destroy()
  return { ok: true }
}

async function abortAfterLoginStart (_phase, username) {
  const socket = await connect()
  socket.write(frame(0, handshakePayload()))
  socket.write(frame(0, writeString(username), randomUuidBytes()))
  socket.destroy()
  return { ok: true }
}

async function abortJoinProbe (phase, username) {
  const abortAfter = {
    login_success: 'login_success',
    configuration: 'registry_sync',
    play_entry: 'first_chunk'
  }[phase]
  const result = await runJoinProbe(username, { RUSTCRAFT_RAW_PROBE_ABORT_AFTER: abortAfter })
  assert.equal(result.aborted, true)
  assert.equal(result.phase, abortAfter)
  return { ok: true }
}

async function runJoinProbe (username, env = {}) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: new URL('.', import.meta.url),
      env: {
        ...process.env,
        RUSTCRAFT_USERNAME: username,
        ...env
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
