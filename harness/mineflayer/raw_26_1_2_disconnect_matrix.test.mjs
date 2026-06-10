import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import crypto from 'node:crypto'
import net from 'node:net'
import test from 'node:test'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)
const host = process.env.VIBECRAFT_HOST ?? '127.0.0.1'
const port = Number(process.env.VIBECRAFT_PORT ?? 25565)
const protocolVersion = Number(process.env.VIBECRAFT_PROTOCOL_VERSION ?? 775)

test('raw 26.1.2 login disconnect matrix cleans up after handshake, login, config, known-packs, finish, and play drops', { timeout: 180_000 }, async () => {
  const phases = [
    ['handshake', abortAfterHandshake],
    ['login_start', abortAfterLoginStart],
    ['login_success', probeAbort('login_success')],
    ['configuration_start', probeAbort('registry_sync')],
    ['known_packs', probeAbort('known_packs')],
    ['finish_configuration', probeAbort('finish_configuration')],
    ['play_entry', probeAbort('first_chunk')]
  ]

  for (const [phase, abort] of phases) {
    const username = `Mx${phase.replaceAll('_', '').slice(0, 7)}${crypto.randomUUID().replaceAll('-', '').slice(0, 5)}`
    const dropped = await abort(username)
    assert.equal(dropped.ok, true)

    const retry = await runJoinProbe(username)
    assert.equal(retry.ok, true, `${phase} retry should reach play`)
    assert.equal(retry.joinState.profile.name, username)
    assert.ok(retry.configPacketCount > 0, `${phase} retry should receive config packets`)
    assert.ok(retry.playPacketCount > 0, `${phase} retry should receive play packets`)
    assert.ok(retry.joinState.initialChunkCount > 0, `${phase} retry should receive initial chunks`)
  }
})

async function abortAfterHandshake () {
  const socket = await connect()
  socket.write(frame(0, handshakePayload()))
  socket.destroy()
  return { ok: true }
}

async function abortAfterLoginStart (username) {
  const socket = await connect()
  socket.write(frame(0, handshakePayload()))
  socket.write(frame(0, writeString(username), randomUuidBytes()))
  socket.destroy()
  return { ok: true }
}

function probeAbort (abortAfter) {
  return async username => {
    const result = await runJoinProbe(username, { VIBECRAFT_RAW_PROBE_ABORT_AFTER: abortAfter })
    assert.equal(result.ok, true)
    assert.equal(result.aborted, true)
    assert.equal(result.phase, abortAfter)
    return { ok: true }
  }
}

async function runJoinProbe (username, env = {}) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: new URL('.', import.meta.url),
      env: {
        ...process.env,
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
