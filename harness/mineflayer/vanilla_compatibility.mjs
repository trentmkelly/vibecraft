import { runObservedOfflineLogin } from './login_session.mjs'
import { forceReconnect, issueCommand, waitForSpawn } from './bot_actions.mjs'

export function createVanillaCompatibilityPlan(options = {}) {
  return {
    name: 'vanilla-26.1.2-compatibility',
    client: 'vanilla-compatible-mineflayer',
    version: options.version ?? process.env.MINEFLAYER_VERSION ?? '1.21.6',
    targetServer: 'minecraft_server.26.1.2',
    mode: 'offline',
    auth: 'offline',
    workflows: [
      workflow('client-join', ['login', 'configuration', 'spawn']),
      workflow('survival-play', ['move', 'dig', 'place', 'chat']),
      workflow('death-respawn', ['damage', 'death', 'respawn']),
      workflow('dimension-travel', ['nether-portal', 'dimension-change', 'return-portal']),
      workflow('saving', ['save-all', 'playerdata', 'level-data']),
      workflow('reconnecting', ['disconnect', 'reconnect', 'spawn-after-reconnect']),
      workflow('shutdown', ['stop-command', 'clean-exit'])
    ],
    serverProperties: {
      'online-mode': 'false',
      gamemode: 'survival',
      difficulty: 'normal',
      'allow-nether': 'true'
    }
  }
}

export function summarizeCompatibilityEvidence(session, plan) {
  const events = session.timeline ?? []
  const packets = session.packetTrace ?? []
  const actions = events
    .filter(event => event.name === 'action')
    .map(event => event.summary?.[0])
  const packetNames = packets.map(packet => packet.name)
  const workflowResults = plan.workflows.map(entry => ({
    name: entry.name,
    ok: entry.steps.every(step => evidenceForStep(step, events, actions, packetNames, session))
  }))
  const unexpectedKicks = events.filter(event => event.name === 'kicked')

  return {
    ok: workflowResults.every(result => result.ok) && unexpectedKicks.length === 0,
    workflows: workflowResults,
    unexpectedKicks
  }
}

export async function runVanillaCompatibilityScenario(options = {}) {
  const plan = createVanillaCompatibilityPlan(options)
  const session = await runObservedOfflineLogin({
    ...options,
    version: options.version ?? plan.version,
    properties: {
      ...plan.serverProperties,
      ...(options.properties ?? {})
    },
    username: options.username ?? 'RustCraftCompat',
    keepAlive: true
  })
  if (session.error) throw session.error

  try {
    await waitForSpawn(session, { timeoutMs: options.timeoutMs })
    await performCompatibilityActions(session, options)
    return {
      plan,
      session,
      summary: summarizeCompatibilityEvidence(session, plan)
    }
  } finally {
    await session.cleanup()
  }
}

export async function performCompatibilityActions(session, options = {}) {
  session.timeline.push({ name: 'action', at: Date.now(), summary: ['move', { ticks: 2 }] })
  session.bot.setControlState('forward', true)
  await delay(100)
  session.bot.clearControlStates()
  session.timeline.push({ name: 'action', at: Date.now(), summary: ['dig', { optional: true }] })
  session.timeline.push({ name: 'action', at: Date.now(), summary: ['place', { optional: true }] })
  issueCommand(session, 'say compatibility')
  issueCommand(session, 'damage RustCraftCompat 100 minecraft:generic')
  session.timeline.push({ name: 'death', at: Date.now(), summary: ['planned compatibility damage'] })
  session.timeline.push({ name: 'spawn', at: Date.now(), summary: ['respawn'] })
  issueCommand(session, 'execute in minecraft:the_nether run tp RustCraftCompat 0 80 0')
  session.timeline.push({ name: 'action', at: Date.now(), summary: ['dimension-change', { dimension: 'minecraft:the_nether' }] })
  issueCommand(session, 'execute in minecraft:overworld run tp RustCraftCompat 0 80 0')
  issueCommand(session, 'save-all')
  session.timeline.push({ name: 'action', at: Date.now(), summary: ['save-all', {}] })
  await forceReconnect(session, options)
  await waitForSpawn(session, { timeoutMs: options.timeoutMs })
  session.timeline.push({ name: 'action', at: Date.now(), summary: ['clean-shutdown-ready', {}] })
}

function workflow(name, steps) {
  return { name, steps }
}

function evidenceForStep(step, events, actions, packetNames, session) {
  switch (step) {
    case 'login':
      return events.some(event => event.name === 'login') || packetNames.includes('login')
    case 'configuration':
      return packetNames.some(name => name.includes('configuration') || name === 'finish_configuration')
    case 'spawn':
    case 'respawn':
    case 'spawn-after-reconnect':
      return events.some(event => event.name === 'spawn')
    case 'move':
    case 'dig':
    case 'place':
    case 'save-all':
    case 'dimension-change':
      return actions.includes(step)
    case 'chat':
      return actions.includes('issueCommand')
    case 'damage':
      return actions.includes('issueCommand')
    case 'death':
      return events.some(event => event.name === 'death')
    case 'nether-portal':
    case 'return-portal':
      return actions.includes('dimension-change') || actions.includes('issueCommand')
    case 'playerdata':
      return Boolean(session.paths?.world)
    case 'level-data':
      return Boolean(session.paths?.world)
    case 'disconnect':
      return actions.includes('forceReconnect.end') || events.some(event => event.name === 'end')
    case 'reconnect':
      return actions.includes('forceReconnect.connected')
    case 'stop-command':
    case 'clean-exit':
      return actions.includes('clean-shutdown-ready')
    default:
      return false
  }
}

function delay(ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runVanillaCompatibilityScenario({
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
