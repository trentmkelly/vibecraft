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

test('raw 26.1.2 offline username validation follows vanilla login rules', { timeout: 180_000 }, async () => {
  const validNames = [
    'Valid_Name',
    'abcdefghijklmnop',
    'CaseProbe',
    'caseprobe',
    'dash-name',
    'period.name'
  ]

  for (const name of validNames) {
    const result = await runJoinProbe(name)
    assert.equal(result.ok, true, `${name} should join`)
    assert.equal(result.joinState.profile.name, name)
    assert.equal(result.joinState.profile.uuid, offlineUuid(name))
  }

  const firstCase = await openPlaySession('CaseDupe')
  const secondCase = await openPlaySession('casedupe')
  firstCase.socket.destroy()
  secondCase.socket.destroy()
  assert.notEqual(firstCase.profile.uuid, secondCase.profile.uuid, 'offline UUIDs should remain case-sensitive')

  for (const name of ['has space', 'newline\nname', 'seventeen_chars__', 'nonasciié', 'delete\u007fname']) {
    const rejected = await attemptLogin(name)
    assert.equal(rejected.accepted, false, `${JSON.stringify(name)} should be rejected before login success`)
    assert.ok(['close', 'reset'].includes(rejected.kind), `${JSON.stringify(name)} should close or reset`)
  }
})

async function openPlaySession (name) {
  const result = await runJoinProbe(name)
  assert.equal(result.ok, true)
  return {
    socket: { destroy () {} },
    profile: result.joinState.profile
  }
}

async function runJoinProbe (name, options = {}) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: new URL('.', import.meta.url),
      env: {
        ...process.env,
        RUSTCRAFT_USERNAME: name,
        RUSTCRAFT_RAW_PROBE_OUTPUT: 'summary',
        RUSTCRAFT_RAW_PROBE_KEEPALIVE_MS: String(options.keepAliveMs ?? 0)
      },
      timeout: 30_000,
      maxBuffer: 1024 * 1024
    }
  )

  return JSON.parse(stdout)
}

async function attemptLogin (name) {
  const socket = net.createConnection({ host, port })
  await Promise.race([
    once(socket, 'connect'),
    once(socket, 'error').then(error => Promise.reject(error))
  ])
  socket.write(frame(0, handshakePayload(2)))
  socket.write(frame(0, writeString(name), randomUuidBytes()))

  const observed = await Promise.race([
    readOneFrame(socket).then(packet => ({ kind: 'packet', packet })),
    once(socket, 'close').then(() => ({ kind: 'close' })),
    once(socket, 'error').then(error => ({ kind: 'reset', error })),
    delay(5_000).then(() => ({ kind: 'timeout' }))
  ])
  socket.destroy()
  return {
    accepted: observed.kind === 'packet' && [2, 3].includes(observed.packet.id),
    kind: observed.kind,
    packetId: observed.packet?.id
  }
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

function writeString (value) {
  const data = Buffer.from(value, 'utf8')
  return Buffer.concat([writeVarInt(data.length), data])
}

async function readOneFrame (socket) {
  let buffer = Buffer.alloc(0)
  return await new Promise((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error('timed out waiting for frame')), 5_000)
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

function offlineUuid (name) {
  const hash = crypto.createHash('md5').update(`OfflinePlayer:${name}`, 'utf8').digest()
  hash[6] = (hash[6] & 0x0f) | 0x30
  hash[8] = (hash[8] & 0x3f) | 0x80
  const hex = hash.toString('hex')
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`
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
