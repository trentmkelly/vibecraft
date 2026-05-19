import { runObservedOfflineLogin } from './login_session.mjs'
import { waitForSpawn } from './bot_actions.mjs'
import { digBlock, placeBlock } from './block_interactions.mjs'
import { clickWindowSlot } from './inventory_scenarios.mjs'

export const FIRST_ACTION_MATRIX_STEPS = [
  'movement',
  'chat',
  'command-suggestion',
  'inventory-click',
  'block-dig',
  'block-place',
  'vanilla-compatible-result'
]

export function createFirstActionMatrixPlan(options = {}) {
  return {
    name: 'mineflayer-first-action-matrix',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'FirstActionBot',
    steps: FIRST_ACTION_MATRIX_STEPS,
    serverProperties: {
      'online-mode': 'false',
      gamemode: 'creative',
      difficulty: 'peaceful',
      'spawn-protection': '0'
    }
  }
}

export function summarizeFirstActionMatrix(session, plan) {
  const actions = (session.timeline ?? [])
    .filter(event => event.name === 'first_action')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return {
    ok: Object.values(steps).every(Boolean),
    steps,
    failures: (session.timeline ?? []).filter(event => ['kicked', 'error'].includes(event.name))
  }
}

export async function runFirstActionMatrix(options = {}) {
  const plan = createFirstActionMatrixPlan(options)
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
    await (options.doMovement ?? doMovement)(session, options)
    recordFirstAction(session, 'movement')

    await (options.doChat ?? doChat)(session, options)
    recordFirstAction(session, 'chat')

    const suggestions = await (options.doCommandSuggestion ?? doCommandSuggestion)(session, options)
    recordFirstAction(session, 'command-suggestion', { suggestions })

    const click = await (options.doInventoryClick ?? doInventoryClick)(session, options)
    recordFirstAction(session, 'inventory-click', click)

    const dig = await (options.doBlockDig ?? doBlockDig)(session, options)
    recordFirstAction(session, 'block-dig', dig)

    const place = await (options.doBlockPlace ?? doBlockPlace)(session, options)
    recordFirstAction(session, 'block-place', place)

    assertVanillaCompatibleResult(session)
    recordFirstAction(session, 'vanilla-compatible-result')

    return {
      plan,
      session,
      summary: summarizeFirstActionMatrix(session, plan)
    }
  } finally {
    await session.cleanup?.()
  }
}

export function recordFirstAction(session, step, details = {}) {
  session.timeline.push({ name: 'first_action', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

async function doMovement(session) {
  session.bot.setControlState('forward', true)
  session.bot.clearControlStates()
}

async function doChat(session) {
  session.bot.chat('first-action-matrix')
}

async function doCommandSuggestion(session) {
  if (typeof session.bot.tabComplete !== 'function') return []
  return await session.bot.tabComplete('/list ')
}

async function doInventoryClick(session, options = {}) {
  const window = options.window ?? session.bot.inventory
  const slot = options.inventorySlot ?? 36
  return await clickWindowSlot(session, window, slot, 0, 0)
}

async function doBlockDig(session, options = {}) {
  const target = options.digTarget ?? session.bot.entity.position.offset(0, -1, 0)
  return await digBlock(session, target, options)
}

async function doBlockPlace(session, options = {}) {
  const target = options.placeTarget ?? session.bot.entity.position.offset(0, -1, 0)
  return await placeBlock(session, target, options)
}

function assertVanillaCompatibleResult(session) {
  const failure = (session.timeline ?? []).find(event => ['kicked', 'error'].includes(event.name))
  if (failure) {
    throw new Error(`First-action matrix failure: ${JSON.stringify(failure.summary ?? [])}`)
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runFirstActionMatrix({
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
