import { once } from 'node:events'
import { runObservedOfflineLogin } from './login_session.mjs'
import { waitForSpawn } from './bot_actions.mjs'

export const POST_LOGIN_READINESS_STEPS = [
  'first-physics-tick',
  'movement-immediate',
  'chat-immediate',
  'command-suggestions-immediate',
  'inventory-window-id',
  'chunk-visibility'
]

export function createPostLoginReadinessPlan(options = {}) {
  return {
    name: 'mineflayer-post-login-readiness',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'ReadyBot',
    steps: POST_LOGIN_READINESS_STEPS,
    serverProperties: {
      'online-mode': 'false',
      gamemode: 'survival',
      difficulty: 'peaceful',
      'spawn-protection': '0'
    }
  }
}

export function summarizePostLoginReadiness(session, plan) {
  const actions = (session.timeline ?? [])
    .filter(event => event.name === 'readiness')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return {
    ok: Object.values(steps).every(Boolean),
    steps,
    unexpectedFailures: (session.timeline ?? []).filter(event => ['kicked', 'error'].includes(event.name))
  }
}

export async function runPostLoginReadiness(options = {}) {
  const plan = createPostLoginReadinessPlan(options)
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
    await (options.waitForFirstPhysicsTick ?? waitForFirstPhysicsTick)(session, options)
    recordReadiness(session, 'first-physics-tick', { noRetrySleep: true })

    await (options.verifyMovement ?? verifyMovementImmediate)(session, options)
    recordReadiness(session, 'movement-immediate', { noRetrySleep: true })

    await (options.verifyChat ?? verifyChatImmediate)(session, options)
    recordReadiness(session, 'chat-immediate', { noRetrySleep: true })

    const suggestions = await (options.verifyCommandSuggestions ?? verifyCommandSuggestionsImmediate)(session, options)
    recordReadiness(session, 'command-suggestions-immediate', { suggestions })

    const windowId = (options.verifyInventoryWindowId ?? verifyInventoryWindowId)(session)
    recordReadiness(session, 'inventory-window-id', { windowId })

    const chunk = (options.verifyChunkVisibility ?? verifyChunkVisibility)(session)
    recordReadiness(session, 'chunk-visibility', chunk)

    return {
      plan,
      session,
      summary: summarizePostLoginReadiness(session, plan)
    }
  } finally {
    await session.cleanup?.()
  }
}

export function recordReadiness(session, step, details = {}) {
  session.timeline.push({ name: 'readiness', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

async function waitForFirstPhysicsTick(session, options = {}) {
  await onceWithTimeout(session.bot, 'physicsTick', options.timeoutMs ?? 30_000)
}

async function verifyMovementImmediate(session) {
  session.bot.setControlState('forward', true)
  await onceWithTimeout(session.bot, 'physicsTick', 5_000)
  session.bot.clearControlStates()
}

async function verifyChatImmediate(session) {
  session.bot.chat('post-login-readiness')
}

async function verifyCommandSuggestionsImmediate(session) {
  if (typeof session.bot.tabComplete !== 'function') return []
  return await session.bot.tabComplete('/list ')
}

function verifyInventoryWindowId(session) {
  const id = session.bot?.inventory?.id
  if (!Number.isInteger(id) || id < 0) {
    throw new Error(`Expected usable player inventory window id, got ${id}`)
  }
  return id
}

function verifyChunkVisibility(session) {
  const position = session.bot?.entity?.position
  const block = position ? session.bot.blockAt(position.offset(0, -1, 0)) : null
  if (!block) throw new Error('Expected a visible block under the bot after first physics tick')
  return {
    position: {
      x: block.position?.x,
      y: block.position?.y,
      z: block.position?.z
    },
    name: block.name ?? null
  }
}

function onceWithTimeout(emitter, event, timeoutMs) {
  return Promise.race([
    once(emitter, event),
    new Promise((_, reject) => setTimeout(() => reject(new Error(`Timed out waiting for ${event}`)), timeoutMs))
  ])
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runPostLoginReadiness({
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
