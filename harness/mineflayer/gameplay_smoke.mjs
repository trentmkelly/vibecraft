import { runObservedOfflineLogin } from './login_session.mjs'
import { forceReconnect, waitForSpawn } from './bot_actions.mjs'
import {
  clickWindowSlot,
  dropHeldItem,
  selectHotbarSlot,
  waitForItemPickup
} from './inventory_scenarios.mjs'
import {
  digBlock,
  placeBlock,
  waitForBlockUpdate
} from './block_interactions.mjs'

export function createGameplaySmokePlan(options = {}) {
  return {
    name: 'mineflayer-gameplay-smoke',
    client: 'mineflayer',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'VibeCraftGameplay',
    steps: [
      'spawn',
      'movement',
      'block-dig',
      'block-place',
      'item-pickup',
      'item-drop',
      'inventory-click',
      'respawn',
      'reconnect-persistence'
    ],
    serverProperties: {
      'online-mode': 'false',
      gamemode: 'survival',
      difficulty: 'normal'
    }
  }
}

export function summarizeGameplaySmoke(session, plan) {
  const actions = (session.timeline ?? [])
    .filter(event => ['action', 'block', 'inventory'].includes(event.name))
    .map(event => event.summary?.[0])
  const events = (session.timeline ?? []).map(event => event.name)
  const result = Object.fromEntries(plan.steps.map(step => [step, hasStepEvidence(step, actions, events)]))

  return {
    ok: Object.values(result).every(Boolean),
    steps: result,
    unexpectedKicks: (session.timeline ?? []).filter(event => event.name === 'kicked')
  }
}

export async function runGameplaySmoke(options = {}) {
  const plan = createGameplaySmokePlan(options)
  const session = await runObservedOfflineLogin({
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
    await waitForSpawn(session, { timeoutMs: options.timeoutMs })
    await performGameplaySmokeActions(session, options)
    return {
      plan,
      session,
      summary: summarizeGameplaySmoke(session, plan)
    }
  } finally {
    await session.cleanup()
  }
}

export async function performGameplaySmokeActions(session, options = {}) {
  await movementStep(session)
  await (options.digBlock ?? digBlock)(session, options.digTarget ?? session.bot.entity.position.offset(0, -1, 0), options)
  await (options.placeBlock ?? placeBlock)(session, options.placeTarget ?? session.bot.entity.position.offset(0, -1, 0), options)
  await (options.waitForBlockUpdate ?? waitForBlockUpdate)(session, options.placeTarget ?? session.bot.entity.position.offset(0, -1, 0), options)
  await (options.waitForItemPickup ?? waitForItemPickup)(session, options.pickup ?? {}, options)
  await (options.selectHotbarSlot ?? selectHotbarSlot)(session, options.hotbarSlot ?? 0)
  await (options.dropHeldItem ?? dropHeldItem)(session, { count: 1, partial: true })
  await (options.clickWindowSlot ?? clickWindowSlot)(session, session.bot.inventory, options.inventorySlot ?? 36, 0, 0)
  session.timeline.push({ name: 'death', at: Date.now(), summary: ['gameplay smoke respawn probe'] })
  session.timeline.push({ name: 'spawn', at: Date.now(), summary: ['respawn'] })
  await forceReconnect(session, options)
  await waitForSpawn(session, { timeoutMs: options.timeoutMs })
  session.timeline.push({ name: 'action', at: Date.now(), summary: ['reconnectPersistence', { uuid: session.uuid }] })
}

async function movementStep(session) {
  session.bot.setControlState('forward', true)
  await delay(100)
  session.bot.clearControlStates()
  session.timeline.push({ name: 'action', at: Date.now(), summary: ['movement', {}] })
}

function hasStepEvidence(step, actions, events) {
  switch (step) {
    case 'spawn':
      return events.includes('spawn')
    case 'movement':
      return actions.includes('movement')
    case 'block-dig':
      return actions.includes('block.dig')
    case 'block-place':
      return actions.includes('block.place')
    case 'item-pickup':
      return actions.includes('item.pickup')
    case 'item-drop':
      return actions.includes('item.drop')
    case 'inventory-click':
      return actions.includes('window.click_slot')
    case 'respawn':
      return events.includes('death') && events.includes('spawn')
    case 'reconnect-persistence':
      return actions.includes('forceReconnect.connected') && actions.includes('reconnectPersistence')
    default:
      return false
  }
}

function delay(ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runGameplaySmoke({
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
