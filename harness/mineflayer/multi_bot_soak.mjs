import { rm } from 'node:fs/promises'
import {
  DEFAULT_TIMEOUT_MS,
  createTempWorld,
  offlineUuid,
  startRustCraft,
  stopServer,
  waitForPort,
  writeOfflineServerFiles
} from './runner.mjs'
import { connectObservedOfflineBot } from './login_session.mjs'

export function createMultiBotSoakPlan(options = {}) {
  const botCount = Number(options.botCount ?? process.env.RUSTCRAFT_SOAK_BOTS ?? 4)
  const cycles = Number(options.cycles ?? process.env.RUSTCRAFT_SOAK_CYCLES ?? 3)
  return {
    name: 'mineflayer-multi-bot-offline-soak',
    client: 'mineflayer',
    mode: 'offline',
    auth: 'offline',
    botCount,
    cycles,
    profiles: Array.from({ length: botCount }, (_, index) => profileForIndex(index)),
    checks: [
      'repeated-offline-joins',
      'leaves',
      'movement-ticks',
      'keepalive-round-trips',
      'chat-broadcasts',
      'reconnects',
      'stable-offline-uuids',
      'no-unexpected-kicks'
    ],
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false',
      'max-players': String(Math.max(botCount + 2, 20))
    }
  }
}

export function summarizeMultiBotSoak(sessions, plan) {
  const joined = sessions.filter(session => session.timeline.some(event => event.name === 'spawn')).length
  const left = sessions.filter(session => session.timeline.some(event => event.name === 'end')).length
  const moved = sessions.filter(session =>
    session.timeline.some(event => event.name === 'action' && event.summary?.[0] === 'multiBotMovementTick')
  ).length
  const chatted = sessions.filter(session =>
    session.timeline.some(event => event.name === 'action' && event.summary?.[0] === 'multiBotChat')
  ).length
  const reconnected = sessions.filter(session =>
    session.timeline.some(event => event.name === 'action' && event.summary?.[0] === 'multiBotReconnect')
  ).length
  const keepAlive = sessions.filter(session =>
    session.packetTrace.some(packet => packet.name === 'keep_alive' && packet.state === 'play')
  ).length
  const kicked = sessions.flatMap(session => session.timeline.filter(event => event.name === 'kicked'))
  const uuidMismatches = sessions.filter(session =>
    session.profile.actualUuid && session.profile.actualUuid !== session.profile.expectedUuid
  )

  return {
    ok:
      joined === plan.botCount &&
      left === plan.botCount &&
      moved === plan.botCount &&
      chatted === plan.botCount &&
      reconnected === plan.botCount &&
      keepAlive === plan.botCount &&
      kicked.length === 0 &&
      uuidMismatches.length === 0,
    joined,
    left,
    moved,
    chatted,
    reconnected,
    keepAlive,
    kicked,
    uuidMismatches
  }
}

export async function runMultiBotSoak(options = {}) {
  const plan = createMultiBotSoakPlan(options)
  const root = options.root ?? await createTempWorld('rustcraft-mf-multibot-')
  await writeOfflineServerFiles(root, {
    ...options,
    properties: {
      ...plan.serverProperties,
      ...(options.properties ?? {})
    }
  })
  const server = startRustCraft({ ...options, root })
  const sessions = []
  let cleaned = false

  const cleanup = async () => {
    if (cleaned) return
    cleaned = true
    for (const session of sessions) session.bot?.end()
    await stopServer(server.child)
    if (options.keepArtifacts !== true) await rm(root, { recursive: true, force: true })
  }

  try {
    await waitForPort(options.port, options.host ?? '127.0.0.1', options.timeoutMs ?? DEFAULT_TIMEOUT_MS)
    for (const profile of plan.profiles) {
      sessions.push(await connectSession(profile, options))
    }
    for (let cycle = 0; cycle < plan.cycles; cycle += 1) {
      for (const session of sessions) {
        await movementTick(session, cycle)
        session.bot.chat(`multi-bot soak ${cycle}`)
        session.timeline.push({ name: 'action', at: Date.now(), summary: ['multiBotChat', { cycle }] })
      }
      await delay(250)
      for (const session of sessions) {
        session.bot.end()
        session.timeline.push({ name: 'action', at: Date.now(), summary: ['multiBotReconnect', { cycle }] })
        const observed = await connectObservedOfflineBot({
          host: options.host ?? '127.0.0.1',
          port: options.port,
          username: session.profile.username,
          version: options.version,
          timeoutMs: options.timeoutMs ?? DEFAULT_TIMEOUT_MS,
          timeline: session.timeline,
          packetTrace: session.packetTrace
        })
        session.bot = observed.bot
        session.profile = observed.profile
      }
    }
    for (const session of sessions) session.bot.end()
    await delay(100)
    return { plan, sessions, summary: summarizeMultiBotSoak(sessions, plan) }
  } finally {
    await cleanup()
  }
}

async function connectSession(profile, options) {
  const timeline = []
  const packetTrace = []
  const observed = await connectObservedOfflineBot({
    host: options.host ?? '127.0.0.1',
    port: options.port,
    username: profile.username,
    version: options.version,
    timeoutMs: options.timeoutMs ?? DEFAULT_TIMEOUT_MS,
    timeline,
    packetTrace
  })
  return {
    bot: observed.bot,
    profile: {
      username: profile.username,
      expectedUuid: profile.expectedUuid,
      actualUuid: observed.profile.actualUuid
    },
    timeline,
    packetTrace
  }
}

async function movementTick(session, cycle) {
  session.bot.setControlState('forward', true)
  await delay(50)
  session.bot.clearControlStates()
  session.timeline.push({ name: 'action', at: Date.now(), summary: ['multiBotMovementTick', { cycle }] })
}

function profileForIndex(index) {
  const username = `RustCraftSoak${index + 1}`
  return {
    username,
    expectedUuid: offlineUuid(username)
  }
}

function delay(ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runMultiBotSoak({
    binary: process.env.RUSTCRAFT_BIN,
    port: Number(process.env.RUSTCRAFT_PORT ?? 25565),
    version: process.env.MINEFLAYER_VERSION,
    timeoutMs: Number(process.env.RUSTCRAFT_TIMEOUT_MS ?? 30_000),
    keepArtifacts: process.env.RUSTCRAFT_KEEP_ARTIFACTS === '1'
  }).then(result => {
    console.log(JSON.stringify({ plan: result.plan, summary: result.summary }, null, 2))
    process.exitCode = result.summary.ok ? 0 : 1
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
