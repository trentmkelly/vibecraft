import { once } from 'node:events'
import { createConnection } from 'node:net'
import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { spawn } from 'node:child_process'

export const DEFAULT_TIMEOUT_MS = 30_000

export function offlineUuid(username) {
  const bytes = md5Bytes(`OfflinePlayer:${username}`)
  bytes[6] = (bytes[6] & 0x0f) | 0x30
  bytes[8] = (bytes[8] & 0x3f) | 0x80
  return formatUuid(bytes)
}

export async function createTempWorld(prefix = 'vibecraft-mf-') {
  const root = await mkdtemp(path.join(tmpdir(), prefix))
  await mkdir(root, { recursive: true })
  return root
}

export async function writeOfflineServerFiles(root, options = {}) {
  await mkdir(root, { recursive: true })
  const properties = {
    'online-mode': 'false',
    'enforce-secure-profile': 'false',
    'level-name': options.levelName ?? 'world',
    'level-seed': String(options.seed ?? 8675309),
    'server-port': String(options.port),
    'motd': options.motd ?? 'VibeCraft Mineflayer harness',
    ...(options.properties ?? {})
  }
  await writeFile(path.join(root, 'eula.txt'), 'eula=true\n')
  await writeFile(
    path.join(root, 'server.properties'),
    Object.entries(properties).map(([key, value]) => `${key}=${value}\n`).join('')
  )
  return properties
}

export function startVibeCraft(options) {
  const args = [
    '--nogui',
    '--universe', options.root,
    '--world', options.levelName ?? 'world',
    '--port', String(options.port)
  ]
  const child = spawn(options.binary, args, {
    cwd: options.root,
    stdio: ['pipe', 'pipe', 'pipe'],
    env: { ...process.env, ...(options.env ?? {}) }
  })
  const logs = []
  child.stdout.setEncoding('utf8')
  child.stderr.setEncoding('utf8')
  child.stdout.on('data', chunk => logs.push({ stream: 'stdout', text: chunk }))
  child.stderr.on('data', chunk => logs.push({ stream: 'stderr', text: chunk }))
  return { child, logs, args }
}

export function startOfficialServer(options) {
  const args = [
    ...(options.javaArgs ?? ['-Xms512M', '-Xmx512M']),
    '-jar',
    options.jar,
    '--nogui'
  ]
  const child = spawn(options.java ?? 'java', args, {
    cwd: options.root,
    stdio: ['pipe', 'pipe', 'pipe'],
    env: { ...process.env, ...(options.env ?? {}) }
  })
  const logs = []
  child.stdout.setEncoding('utf8')
  child.stderr.setEncoding('utf8')
  child.stdout.on('data', chunk => logs.push({ stream: 'stdout', text: chunk }))
  child.stderr.on('data', chunk => logs.push({ stream: 'stderr', text: chunk }))
  return { child, logs, args }
}

export async function waitForPort(port, host = '127.0.0.1', timeoutMs = DEFAULT_TIMEOUT_MS) {
  const start = Date.now()
  let lastError
  while (Date.now() - start < timeoutMs) {
    try {
      await connectOnce(port, host)
      return
    } catch (error) {
      lastError = error
      await delay(100)
    }
  }
  const message = lastError ? lastError.message : 'port did not open'
  throw new Error(`Timed out waiting for ${host}:${port}: ${message}`)
}

export async function connectOfflineBot(options) {
  const mineflayer = await import('mineflayer')
  const events = []
  const bot = mineflayer.createBot({
    host: options.host ?? '127.0.0.1',
    port: options.port,
    username: options.username,
    version: options.version,
    auth: 'offline'
  })
  captureBotEvents(bot, events)
  await onceWithTimeout(bot, 'spawn', options.timeoutMs ?? DEFAULT_TIMEOUT_MS)
  return {
    bot,
    events,
    profile: {
      username: options.username,
      expectedUuid: offlineUuid(options.username),
      actualUuid: bot.player?.uuid ?? null
    }
  }
}

export async function runOfflineLoginScenario(options) {
  const root = options.root ?? await createTempWorld()
  await writeOfflineServerFiles(root, options)
  const server = startScenarioServer({ ...options, root })
  try {
    await waitForPort(options.port, options.host, options.timeoutMs)
    const session = await connectOfflineBot({
      host: options.host,
      port: options.port,
      username: options.username ?? 'VibeCraftBot',
      version: options.version,
      timeoutMs: options.timeoutMs
    })
    return {
      root,
      server,
      session,
      artifacts: await collectArtifacts(root, session.events, server.logs)
    }
  } catch (error) {
    return {
      root,
      server,
      error,
      artifacts: await collectArtifacts(root, [], server.logs)
    }
  } finally {
    if (options.keepAlive !== true) {
      await stopServer(server.child)
      if (options.keepArtifacts !== true) await rm(root, { recursive: true, force: true })
    }
  }
}

export async function runParityScenario(options) {
  const basePort = options.port
  const official = await runOfflineLoginScenario({
    ...options,
    root: options.officialRoot ?? await createTempWorld('vibecraft-mf-official-'),
    serverKind: 'official',
    port: basePort,
    keepArtifacts: true
  })
  const rebuilt = await runOfflineLoginScenario({
    ...options,
    root: options.rebuiltRoot ?? await createTempWorld('vibecraft-mf-vibecraft-'),
    serverKind: 'vibecraft',
    port: options.rebuiltPort ?? basePort + 1,
    keepArtifacts: true
  })
  const diff = diffArtifacts(
    normalizeArtifacts(official.artifacts, { root: official.root, port: basePort }),
    normalizeArtifacts(rebuilt.artifacts, { root: rebuilt.root, port: options.rebuiltPort ?? basePort + 1 })
  )
  if (options.keepArtifacts !== true) {
    await rm(official.root, { recursive: true, force: true })
    await rm(rebuilt.root, { recursive: true, force: true })
  }
  return { official, rebuilt, diff, equivalent: diff.length === 0 }
}

export async function collectArtifacts(root, events, logs) {
  return {
    root,
    events: [...events],
    logs: logs.map(entry => ({ ...entry })),
    eula: await readOptional(path.join(root, 'eula.txt')),
    serverProperties: await readOptional(path.join(root, 'server.properties'))
  }
}

export function normalizeArtifacts(artifacts, options = {}) {
  return {
    events: artifacts.events.map(normalizeEvent),
    logs: normalizeLogs(artifacts.logs, options),
    serverProperties: normalizeProperties(artifacts.serverProperties, options),
    eula: artifacts.eula
  }
}

export function diffArtifacts(left, right) {
  const diffs = []
  compareJson(diffs, 'events', left.events, right.events)
  compareJson(diffs, 'logs', left.logs, right.logs)
  compareJson(diffs, 'serverProperties', left.serverProperties, right.serverProperties)
  compareJson(diffs, 'eula', left.eula, right.eula)
  return diffs
}

function startScenarioServer(options) {
  if (options.serverKind === 'official') {
    return startOfficialServer(options)
  }
  return startVibeCraft(options)
}

export async function stopServer(child) {
  if (child.exitCode !== null || child.signalCode !== null) return
  child.stdin?.once('error', () => {})
  child.stdin?.write('stop\n', () => {})
  const exited = onceWithTimeout(child, 'exit', 3_000).catch(async () => {
    child.kill('SIGTERM')
    await onceWithTimeout(child, 'exit', 3_000).catch(() => child.kill('SIGKILL'))
  })
  await exited
}

function normalizeEvent(event) {
  return {
    name: event.name,
    summary: event.summary
  }
}

function normalizeLogs(logs, options = {}) {
  return logs
    .flatMap(entry => entry.text.split(/\r?\n/).filter(Boolean).map(line => ({
      stream: entry.stream,
      text: normalizeVolatileText(line, options)
    })))
    .filter(entry => !/^\s*$/.test(entry.text))
}

function normalizeProperties(body, options = {}) {
  if (body == null) return null
  return body
    .split(/\r?\n/)
    .filter(Boolean)
    .map(line => normalizeVolatileText(line, options))
    .sort()
    .join('\n')
}

function normalizeVolatileText(text, options = {}) {
  let output = text
  if (options.root) output = output.replaceAll(options.root, '<run-dir>')
  if (options.port) output = output.replaceAll(String(options.port), '<port>')
  return output
    .replace(/\b\d{4}-\d{2}-\d{2}[T ][0-9:.Z+-]+/g, '<timestamp>')
    .replace(/vibecraft-mf-(official|vibecraft|files|spawn)-[A-Za-z0-9._-]+/g, 'vibecraft-mf-<run>')
}

function compareJson(diffs, path, left, right) {
  const leftJson = JSON.stringify(left)
  const rightJson = JSON.stringify(right)
  if (leftJson !== rightJson) {
    diffs.push({ path, official: left, rebuilt: right })
  }
}

export function captureBotEvents(bot, events) {
  for (const name of ['login', 'spawn', 'kicked', 'end', 'error', 'death', 'message']) {
    bot.on(name, (...args) => {
      events.push({
        name,
        at: Date.now(),
        summary: args.map(summarizeEventArg)
      })
    })
  }
  bot._client?.on('packet', (_data, meta) => {
    events.push({ name: 'packet', at: Date.now(), summary: [meta?.name ?? 'unknown'] })
  })
}

function summarizeEventArg(arg) {
  if (arg == null) return arg
  if (typeof arg === 'string' || typeof arg === 'number' || typeof arg === 'boolean') return arg
  if (typeof arg.toString === 'function') return arg.toString()
  return Object.prototype.toString.call(arg)
}

function connectOnce(port, host) {
  return new Promise((resolve, reject) => {
    const socket = createConnection({ port, host })
    socket.once('connect', () => {
      socket.destroy()
      resolve()
    })
    socket.once('error', reject)
    socket.setTimeout(1_000, () => {
      socket.destroy()
      reject(new Error('socket timeout'))
    })
  })
}

function onceWithTimeout(emitter, event, timeoutMs) {
  return Promise.race([
    once(emitter, event),
    new Promise((_, reject) => {
      setTimeout(() => reject(new Error(`Timed out waiting for ${event}`)), timeoutMs)
    })
  ])
}

function delay(ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

async function readOptional(file) {
  try {
    return await readFile(file, 'utf8')
  } catch {
    return null
  }
}

function md5Bytes(input) {
  const leftRotate = (value, amount) => ((value << amount) | (value >>> (32 - amount))) >>> 0
  const message = Buffer.from(input, 'utf8')
  const bitLength = message.length * 8
  const withPadding = Buffer.concat([
    message,
    Buffer.from([0x80]),
    Buffer.alloc((56 - ((message.length + 1) % 64) + 64) % 64),
    Buffer.alloc(8)
  ])
  withPadding.writeUInt32LE(bitLength >>> 0, withPadding.length - 8)
  withPadding.writeUInt32LE(Math.floor(bitLength / 0x100000000), withPadding.length - 4)

  let a0 = 0x67452301
  let b0 = 0xefcdab89
  let c0 = 0x98badcfe
  let d0 = 0x10325476
  const s = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
    5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
    4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
    6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21
  ]
  const k = Array.from({ length: 64 }, (_, i) => Math.floor(Math.abs(Math.sin(i + 1)) * 0x100000000) >>> 0)

  for (let offset = 0; offset < withPadding.length; offset += 64) {
    const m = Array.from({ length: 16 }, (_, i) => withPadding.readUInt32LE(offset + i * 4))
    let a = a0; let b = b0; let c = c0; let d = d0
    for (let i = 0; i < 64; i++) {
      let f; let g
      if (i < 16) {
        f = (b & c) | (~b & d); g = i
      } else if (i < 32) {
        f = (d & b) | (~d & c); g = (5 * i + 1) % 16
      } else if (i < 48) {
        f = b ^ c ^ d; g = (3 * i + 5) % 16
      } else {
        f = c ^ (b | ~d); g = (7 * i) % 16
      }
      const temp = d
      d = c
      c = b
      b = (b + leftRotate((a + f + k[i] + m[g]) >>> 0, s[i])) >>> 0
      a = temp
    }
    a0 = (a0 + a) >>> 0
    b0 = (b0 + b) >>> 0
    c0 = (c0 + c) >>> 0
    d0 = (d0 + d) >>> 0
  }
  const output = Buffer.alloc(16)
  output.writeUInt32LE(a0, 0)
  output.writeUInt32LE(b0, 4)
  output.writeUInt32LE(c0, 8)
  output.writeUInt32LE(d0, 12)
  return [...output]
}

function formatUuid(bytes) {
  const hex = bytes.map(byte => byte.toString(16).padStart(2, '0')).join('')
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const repoRoot = path.resolve(new URL('../..', import.meta.url).pathname)
  const options = {
    binary: process.env.VIBECRAFT_BIN ?? path.join(repoRoot, 'target', 'debug', 'vibecraft'),
    port: Number(process.env.VIBECRAFT_PORT ?? 25565),
    username: process.env.VIBECRAFT_BOT ?? 'VibeCraftBot',
    version: process.env.MINEFLAYER_VERSION,
    keepArtifacts: process.env.KEEP_ARTIFACTS === '1'
  }
  const result = await runOfflineLoginScenario(options)
  if (result.error) {
    console.error(result.error)
    console.error(JSON.stringify(result.artifacts, null, 2))
    process.exitCode = 1
  } else {
    result.session.bot.end()
    console.log(JSON.stringify({
      root: result.root,
      profile: result.session.profile,
      events: result.session.events.map(event => event.name)
    }, null, 2))
  }
}
