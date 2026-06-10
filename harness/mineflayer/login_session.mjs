import { once } from 'node:events'
import { rm } from 'node:fs/promises'
import path from 'node:path'
import {
  DEFAULT_TIMEOUT_MS,
  collectArtifacts,
  createTempWorld,
  offlineUuid,
  startOfficialServer,
  startVibeCraft,
  stopServer,
  waitForPort,
  writeOfflineServerFiles
} from './runner.mjs'

export async function runObservedOfflineLogin(options = {}) {
  const root = options.root ?? await createTempWorld('vibecraft-mf-login-')
  const paths = loginSessionPaths(root, options.levelName ?? 'world')
  let server
  let bot
  let cleaned = false
  const timeline = []
  const packetTrace = []

  await writeOfflineServerFiles(root, options)
  server = startLoginServer({ ...options, root })
  const serverLogStart = server.logs.length

  const cleanup = async () => {
    if (cleaned) return
    cleaned = true
    if (bot && options.keepBot !== true) bot.end()
    if (options.keepAlive !== true) await stopServer(server.child)
    if (options.keepArtifacts !== true) await rm(root, { recursive: true, force: true })
  }

  try {
    await (options.waitForReady ?? waitForPort)(options.port, options.host ?? '127.0.0.1', options.timeoutMs ?? DEFAULT_TIMEOUT_MS)
    const observed = await (options.connectBot ?? connectObservedOfflineBot)({
      host: options.host ?? '127.0.0.1',
      port: options.port,
      username: options.username ?? 'VibeCraftBot',
      version: options.version,
      timeoutMs: options.timeoutMs ?? DEFAULT_TIMEOUT_MS,
      timeline,
      packetTrace
    })
    bot = observed.bot
    return buildSession({
      root,
      paths,
      endpoint: {
        host: options.host ?? '127.0.0.1',
        port: options.port,
        version: options.version
      },
      server,
      bot,
      timeline,
      packetTrace,
      profile: observed.profile,
      serverLogStart,
      cleanup
    })
  } catch (error) {
    const session = buildSession({
      root,
      paths,
      endpoint: {
        host: options.host ?? '127.0.0.1',
        port: options.port,
        version: options.version
      },
      server,
      bot,
      timeline,
      packetTrace,
      profile: {
        username: options.username ?? 'VibeCraftBot',
        expectedUuid: offlineUuid(options.username ?? 'VibeCraftBot'),
        actualUuid: null
      },
      serverLogStart,
      cleanup
    })
    session.error = error
    return session
  } finally {
    if (options.autoCleanup === true) await cleanup()
  }
}

export async function connectObservedOfflineBot(options) {
  const mineflayer = await import('mineflayer')
  const timeline = options.timeline ?? []
  const packetTrace = options.packetTrace ?? []
  const bot = mineflayer.createBot({
    host: options.host ?? '127.0.0.1',
    port: options.port,
    username: options.username,
    version: options.version,
    auth: 'offline'
  })
  captureObservedBotEvents(bot, timeline, packetTrace)
  await onceWithTimeout(bot, 'spawn', options.timeoutMs ?? DEFAULT_TIMEOUT_MS)
  return {
    bot,
    timeline,
    packetTrace,
    profile: {
      username: options.username,
      expectedUuid: offlineUuid(options.username),
      actualUuid: bot.player?.uuid ?? null
    }
  }
}

export function captureObservedBotEvents(bot, timeline, packetTrace) {
  for (const name of ['login', 'spawn', 'kicked', 'end', 'error', 'death', 'message']) {
    bot.on(name, (...args) => {
      timeline.push({
        name,
        at: Date.now(),
        summary: args.map(summarizeEventArg)
      })
    })
  }
  bot._client?.on('packet', (data, meta) => {
    const entry = {
      name: meta?.name ?? 'unknown',
      state: meta?.state,
      at: Date.now(),
      keys: data && typeof data === 'object' ? Object.keys(data).sort() : []
    }
    packetTrace.push(entry)
    timeline.push({ name: 'packet', at: entry.at, summary: [entry.name, entry.state].filter(Boolean) })
  })
}

export function loginSessionPaths(root, levelName = 'world') {
  return {
    root,
    eula: path.join(root, 'eula.txt'),
    serverProperties: path.join(root, 'server.properties'),
    world: path.join(root, levelName)
  }
}

function buildSession({ root, paths, endpoint, server, bot, timeline, packetTrace, profile, serverLogStart, cleanup }) {
  const session = {
    root,
    paths,
    endpoint,
    server,
    bot,
    profile,
    uuid: profile.actualUuid ?? profile.expectedUuid,
    timeline,
    events: timeline,
    packetTrace,
    get serverLogs() {
      return server.logs.slice(serverLogStart).map(entry => ({ ...entry }))
    },
    cleanup,
    handles: {
      cleanup,
      stopServer: () => stopServer(server.child),
      endBot: () => bot?.end(),
      removeArtifacts: () => rm(root, { recursive: true, force: true })
    },
    collectArtifacts: () => collectArtifacts(root, timeline, server.logs.slice(serverLogStart))
  }
  return session
}

function startLoginServer(options) {
  if (options.startServer) return options.startServer(options)
  if (options.serverKind === 'official') return startOfficialServer(options)
  return startVibeCraft(options)
}

function summarizeEventArg(arg) {
  if (arg == null) return arg
  if (typeof arg === 'string' || typeof arg === 'number' || typeof arg === 'boolean') return arg
  if (typeof arg.toString === 'function') return arg.toString()
  return Object.prototype.toString.call(arg)
}

function onceWithTimeout(emitter, event, timeoutMs) {
  return Promise.race([
    once(emitter, event),
    new Promise((_, reject) => {
      setTimeout(() => reject(new Error(`Timed out waiting for ${event}`)), timeoutMs)
    })
  ])
}
