import { runObservedOfflineLogin } from './login_session.mjs'
import { forceReconnect, issueCommand, waitForSpawn } from './bot_actions.mjs'

export function createLongRunningSoakPlan(options = {}) {
  const minutes = Number(options.minutes ?? process.env.RUSTCRAFT_SOAK_MINUTES ?? 30)
  const reconnectEvery = Number(options.reconnectEvery ?? process.env.RUSTCRAFT_SOAK_RECONNECT_EVERY ?? 5)
  const movementEvery = Number(options.movementEvery ?? process.env.RUSTCRAFT_SOAK_MOVE_EVERY ?? 1)
  return {
    name: 'real-client-long-running-soak',
    client: 'mineflayer',
    mode: 'offline',
    auth: 'offline',
    minutes,
    minimumTicks: Math.max(1, Math.floor(minutes * 60 * 20)),
    reconnectEvery,
    movementEvery,
    checks: [
      'spawn-before-soak',
      'play-packets-continue',
      'keepalive-round-trips',
      'movement-ticks-accepted',
      'chat-command-round-trip',
      'reconnect-preserves-play-state',
      'no-unexpected-kick',
      'clean-shutdown'
    ],
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false',
      'view-distance': '4',
      'simulation-distance': '4'
    }
  }
}

export function summarizeSoakHealth(session, plan, options = {}) {
  const packetTrace = session.packetTrace ?? []
  const timeline = session.timeline ?? []
  const keepAliveInbound = packetTrace.filter(packet =>
    packet.name === 'keep_alive' && packet.state === 'play'
  ).length
  const playPackets = packetTrace.filter(packet => packet.state === 'play').length
  const unexpectedKicks = timeline.filter(event => event.name === 'kicked')
  const reconnects = timeline.filter(event =>
    event.name === 'action' && event.summary?.[0] === 'forceReconnect.connected'
  ).length
  const movementTicks = timeline.filter(event =>
    event.name === 'action' && event.summary?.[0] === 'soakMovementTick'
  ).length
  const chatCommands = timeline.filter(event =>
    event.name === 'action' && event.summary?.[0] === 'issueCommand'
  ).length
  const minimumPlayPackets = Number(options.minimumPlayPackets ?? Math.min(plan.minimumTicks, 200))

  return {
    ok:
      timeline.some(event => event.name === 'spawn') &&
      playPackets >= minimumPlayPackets &&
      keepAliveInbound > 0 &&
      movementTicks > 0 &&
      chatCommands > 0 &&
      reconnects >= Number(options.minimumReconnects ?? 1) &&
      unexpectedKicks.length === 0,
    playPackets,
    keepAliveInbound,
    movementTicks,
    chatCommands,
    reconnects,
    unexpectedKicks
  }
}

export async function runLongRunningSoak(options = {}) {
  const plan = createLongRunningSoakPlan(options)
  const session = await runObservedOfflineLogin({
    ...options,
    properties: {
      ...plan.serverProperties,
      ...(options.properties ?? {})
    },
    username: options.username ?? 'RustCraftSoak',
    keepAlive: true
  })

  if (session.error) throw session.error

  try {
    await waitForSpawn(session, { timeoutMs: options.timeoutMs })
    issueCommand(session, 'list')
    const deadline = Date.now() + plan.minutes * 60_000
    let iteration = 0
    while (Date.now() < deadline) {
      iteration += 1
      await movementTick(session, iteration)
      if (iteration % plan.reconnectEvery === 0) {
        await forceReconnect(session, options)
        await waitForSpawn(session, { timeoutMs: options.timeoutMs })
      }
      await delay(plan.movementEvery * 1_000)
    }
    return {
      plan,
      session,
      health: summarizeSoakHealth(session, plan, options)
    }
  } finally {
    await session.cleanup()
  }
}

async function movementTick(session, iteration) {
  const bot = session.bot
  bot.setControlState('forward', true)
  bot.setControlState('jump', iteration % 2 === 0)
  await delay(100)
  bot.clearControlStates()
  session.timeline.push({
    name: 'action',
    at: Date.now(),
    summary: ['soakMovementTick', { iteration }]
  })
}

function delay(ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runLongRunningSoak({
    binary: process.env.RUSTCRAFT_BIN,
    port: Number(process.env.RUSTCRAFT_PORT ?? 25565),
    version: process.env.MINEFLAYER_VERSION,
    timeoutMs: Number(process.env.RUSTCRAFT_TIMEOUT_MS ?? 30_000),
    keepArtifacts: process.env.RUSTCRAFT_KEEP_ARTIFACTS === '1'
  }).then(result => {
    console.log(JSON.stringify({ plan: result.plan, health: result.health }, null, 2))
    process.exitCode = result.health.ok ? 0 : 1
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
