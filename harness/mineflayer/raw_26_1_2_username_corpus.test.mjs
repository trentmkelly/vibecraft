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

const corpus = [
  { name: 'A', accepted: true },
  { name: 'Name_123', accepted: true },
  { name: 'abcdefghijklmnop', accepted: true },
  { name: 'dash-name', accepted: true },
  { name: 'period.name', accepted: true },
  { name: 'tilde~name', accepted: true },
  { name: 'bang!name', accepted: true },
  { name: 'CaseCorpus', accepted: true },
  { name: 'casecorpus', accepted: true },
  { name: 'has space', accepted: false, disconnect: 'transport-close-without-login-success' },
  { name: 'tab\tname', accepted: false, disconnect: 'transport-close-without-login-success' },
  { name: 'newline\nname', accepted: false, disconnect: 'transport-close-without-login-success' },
  { name: 'seventeen_chars__', accepted: false, disconnect: 'transport-close-without-login-success' },
  { name: 'nonasciié', accepted: false, disconnect: 'transport-close-without-login-success' },
  { name: 'delete\u007fname', accepted: false, disconnect: 'transport-close-without-login-success' }
]

test('raw 26.1.2 offline username corpus preserves accepted identities and rejected disconnect shape', { timeout: 240_000 }, async () => {
  const acceptedResults = new Map()

  for (const entry of corpus) {
    if (entry.accepted) {
      const result = await runJoinProbe(entry.name)
      assert.equal(result.ok, true, `${entry.name} should join`)
      assert.equal(result.joinState.profile.name, entry.name)
      assert.equal(result.joinState.profile.uuid, offlineUuid(entry.name))
      acceptedResults.set(entry.name, result.joinState.profile.uuid)
    } else {
      const result = await attemptLogin(entry.name)
      assert.equal(result.accepted, false, `${JSON.stringify(entry.name)} should not reach login success`)
      assert.equal(result.disconnect, entry.disconnect, `${JSON.stringify(entry.name)} disconnect shape drifted`)
    }
  }

  assert.notEqual(
    acceptedResults.get('CaseCorpus'),
    acceptedResults.get('casecorpus'),
    'offline UUID derivation must preserve case-sensitive display names'
  )
})

async function runJoinProbe (name) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: new URL('.', import.meta.url),
      env: {
        ...process.env,
        RUSTCRAFT_USERNAME: name,
        RUSTCRAFT_RAW_PROBE_OUTPUT: 'summary'
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

  const accepted = observed.kind === 'packet' && [2, 3].includes(observed.packet.id)
  return {
    accepted,
    packetId: observed.packet?.id,
    disconnect: accepted ? 'login-success' : classifyDisconnect(observed)
  }
}

function classifyDisconnect (observed) {
  if (observed.kind === 'packet') return `login-packet-${observed.packet.id}`
  if (observed.kind === 'close' || observed.kind === 'reset') return 'transport-close-without-login-success'
  return observed.kind
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
