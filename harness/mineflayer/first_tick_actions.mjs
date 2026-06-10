import { runObservedOfflineLogin } from './login_session.mjs'
import { waitForSpawn } from './bot_actions.mjs'

export const FIRST_TICK_ACTION_STEPS = [
  'movement',
  'chat',
  'command-suggestion',
  'inventory-window',
  'block-look',
  'no-race-disconnect'
]

export function createFirstTickActionPlan(options = {}) {
  return {
    name: 'mineflayer-first-tick-actions',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'FirstTickBot',
    steps: FIRST_TICK_ACTION_STEPS,
    serverProperties: {
      'online-mode': 'false',
      gamemode: 'survival',
      difficulty: 'peaceful',
      'spawn-protection': '0'
    }
  }
}

export function summarizeFirstTickActions(session, plan) {
  const actions = (session.timeline ?? [])
    .filter(event => event.name === 'first_tick')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return {
    ok: Object.values(steps).every(Boolean),
    steps,
    failures: (session.timeline ?? []).filter(event => ['kicked', 'error'].includes(event.name))
  }
}

export async function runFirstTickActions(options = {}) {
  const plan = createFirstTickActionPlan(options)
  const session = await (options.runObservedOfflineLogin ?? runObservedOfflineLogin)({
    ...options,
    username: plan.username,
    properties: {
      ...plan.serverProperties,
      ...(options.properties ?? {})
    },
    keepAlive: true
  })
  if (session.error) throw session.error

  try {
    await (options.waitForSpawn ?? waitForSpawn)(session, { timeoutMs: options.timeoutMs })
    await (options.sendMovement ?? sendMovement)(session, options)
    recordFirstTick(session, 'movement')

    await (options.sendChat ?? sendChat)(session, options)
    recordFirstTick(session, 'chat')

    const suggestions = await (options.sendCommandSuggestion ?? sendCommandSuggestion)(session, options)
    recordFirstTick(session, 'command-suggestion', { suggestions })

    const inventory = await (options.sendInventoryAction ?? sendInventoryAction)(session, options)
    recordFirstTick(session, 'inventory-window', inventory)

    const blockLook = await (options.sendBlockLook ?? sendBlockLook)(session, options)
    recordFirstTick(session, 'block-look', blockLook)

    assertNoRaceDisconnect(session)
    recordFirstTick(session, 'no-race-disconnect')

    return {
      plan,
      session,
      summary: summarizeFirstTickActions(session, plan)
    }
  } finally {
    await session.cleanup?.()
  }
}

export function recordFirstTick(session, step, details = {}) {
  session.timeline.push({ name: 'first_tick', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

async function sendMovement(session) {
  session.bot.setControlState('forward', true)
  session.bot.clearControlStates()
}

async function sendChat(session) {
  session.bot.chat('first-tick-chat')
}

async function sendCommandSuggestion(session) {
  if (typeof session.bot.tabComplete !== 'function') return []
  return await session.bot.tabComplete('/list ')
}

async function sendInventoryAction(session) {
  const id = session.bot?.inventory?.id
  if (!Number.isInteger(id) || id < 0) {
    throw new Error(`Expected usable inventory window id, got ${id}`)
  }
  session.bot._client?.write?.('close_window', { windowId: id })
  return { windowId: id }
}

async function sendBlockLook(session) {
  const position = session.bot?.entity?.position
  if (!position) throw new Error('Missing bot position for block-look action')
  const target = position.offset(0, -1, 0)
  if (typeof session.bot.lookAt === 'function') await session.bot.lookAt(target, true)
  return { x: target.x, y: target.y, z: target.z }
}

function assertNoRaceDisconnect(session) {
  const failure = (session.timeline ?? []).find(event => ['kicked', 'error'].includes(event.name))
  if (failure) throw new Error(`First-tick action race failure: ${JSON.stringify(failure.summary ?? [])}`)
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runFirstTickActions({
    binary: process.env.VIBECRAFT_BIN,
    port: Number(process.env.VIBECRAFT_PORT ?? 25565),
    version: process.env.MINEFLAYER_VERSION,
    timeoutMs: Number(process.env.VIBECRAFT_TIMEOUT_MS ?? 30_000),
    keepArtifacts: process.env.VIBECRAFT_KEEP_ARTIFACTS === '1'
  }).then(result => {
    console.log(JSON.stringify({ plan: result.plan, summary: result.summary }, null, 2))
    process.exitCode = result.summary.ok ? 0 : 1
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
