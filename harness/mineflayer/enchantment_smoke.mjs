import { runObservedOfflineLogin } from './login_session.mjs'
import { waitForSpawn } from './bot_actions.mjs'

// Enchantment smoke test: join offline mode, receive enchanted items via server
// commands, then verify the client stays connected and inventory decoding succeeds.

export function createEnchantmentSmokePlan(options = {}) {
  return {
    name: 'mineflayer-enchantment-smoke',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'EnchantBot',
    serverProperties: {
      'online-mode': 'false',
      gamemode: 'creative',
      difficulty: 'peaceful',
      'spawn-protection': '0'
    },
    steps: [
      'give-enchanted-sword-no-registry-error',
      'give-enchanted-pickaxe-no-registry-error',
      'give-enchanted-boots-no-registry-error',
      'tooltip-decode-no-component-error',
      'enchant-tag-lookup-no-missing-tag'
    ]
  }
}

export function summarizeEnchantmentSmoke(session, plan) {
  const actions = (session.timeline ?? [])
    .filter(e => e.name === 'enchant')
    .map(e => e.summary?.[0])
  const result = Object.fromEntries(
    plan.steps.map(step => [step, actions.includes(stepToAction(step))])
  )
  return {
    ok: Object.values(result).every(Boolean),
    steps: result
  }
}

function stepToAction(step) {
  const map = {
    'give-enchanted-sword-no-registry-error': 'enchant.give.sword',
    'give-enchanted-pickaxe-no-registry-error': 'enchant.give.pickaxe',
    'give-enchanted-boots-no-registry-error': 'enchant.give.boots',
    'tooltip-decode-no-component-error': 'enchant.tooltip.decode_ok',
    'enchant-tag-lookup-no-missing-tag': 'enchant.tag.lookup_ok'
  }
  return map[step] ?? step
}

export function recordEnchantEvent(session, action, details = {}) {
  session.timeline.push({ name: 'enchant', at: Date.now(), summary: [action, details] })
  return { action, ...details }
}

export async function runEnchantmentSmoke(options = {}) {
  const plan = createEnchantmentSmokePlan(options)
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
    for (const item of ENCHANTED_SMOKE_ITEMS) {
      const command = buildGiveCommandForTarget(item, session.profile?.username ?? plan.username)
      await (options.sendCommand ?? sendConsoleCommand)(session, command)
      const evidence = await (options.waitForInventoryItem ?? waitForInventoryItem)(
        session,
        item,
        { timeoutMs: options.timeoutMs }
      )
      recordEnchantEvent(session, item.action, {
        command,
        item: item.itemId,
        enchantments: item.enchantments,
        evidence
      })
      assertNoClientDecodeFailure(session)
    }

    recordEnchantEvent(session, 'enchant.tooltip.decode_ok', {
      inventoryItems: summarizeInventoryItems(session)
    })
    recordEnchantEvent(session, 'enchant.tag.lookup_ok', {
      enchantments: ENCHANTED_SMOKE_ITEMS.flatMap(item => item.enchantments.map(e => e.id))
    })

    return {
      plan,
      session,
      summary: summarizeEnchantmentSmoke(session, plan)
    }
  } finally {
    await session.cleanup?.()
  }
}

// Enchanted items used in smoke tests
export const ENCHANTED_SMOKE_ITEMS = [
  {
    step: 'give-enchanted-sword-no-registry-error',
    action: 'enchant.give.sword',
    itemId: 'minecraft:diamond_sword',
    enchantments: [
      { id: 'minecraft:sharpness', level: 5 },
      { id: 'minecraft:looting', level: 3 },
      { id: 'minecraft:unbreaking', level: 3 }
    ]
  },
  {
    step: 'give-enchanted-pickaxe-no-registry-error',
    action: 'enchant.give.pickaxe',
    itemId: 'minecraft:diamond_pickaxe',
    enchantments: [
      { id: 'minecraft:efficiency', level: 5 },
      { id: 'minecraft:fortune', level: 3 },
      { id: 'minecraft:mending', level: 1 }
    ]
  },
  {
    step: 'give-enchanted-boots-no-registry-error',
    action: 'enchant.give.boots',
    itemId: 'minecraft:diamond_boots',
    enchantments: [
      { id: 'minecraft:feather_falling', level: 4 },
      { id: 'minecraft:protection', level: 4 },
      { id: 'minecraft:depth_strider', level: 3 }
    ]
  }
]

export function buildGiveCommand(item) {
  return buildGiveCommandForTarget(item, '@s')
}

export function buildGiveCommandForTarget(item, target) {
  const levels = item.enchantments
    .map(e => `"${e.id}":${e.level}`)
    .join(',')
  return `/give ${target} ${item.itemId}[minecraft:enchantments={levels:{${levels}}}] 1`
}

function sendConsoleCommand(session, command) {
  const line = command.startsWith('/') ? command.slice(1) : command
  session.server?.child?.stdin?.write(`${line}\n`)
  recordEnchantEvent(session, 'enchant.console.command', { command })
}

async function waitForInventoryItem(session, item, options = {}) {
  const wantedName = item.itemId.split(':').pop()
  const timeoutMs = options.timeoutMs ?? 30_000
  const start = Date.now()
  while (Date.now() - start < timeoutMs) {
    const stack = session.bot?.inventory?.items?.().find(candidate => {
      return candidate.name === wantedName || candidate.type === item.itemId || candidate.displayName === item.itemId
    })
    if (stack) return summarizeItem(stack)
    await delay(100)
  }
  throw new Error(`Timed out waiting for ${item.itemId} in bot inventory`)
}

function assertNoClientDecodeFailure(session) {
  const badEvent = (session.timeline ?? []).find(event => {
    if (!['kicked', 'error', 'end'].includes(event.name)) return false
    const text = JSON.stringify(event.summary ?? [])
    return /registry|tag|tooltip|component|decode|missing|disconnect|kicked/i.test(text) || event.name !== 'end'
  })
  if (badEvent) {
    throw new Error(`Enchantment smoke saw client failure: ${JSON.stringify(badEvent.summary ?? [])}`)
  }
}

function summarizeInventoryItems(session) {
  return session.bot?.inventory?.items?.().map(summarizeItem) ?? []
}

function summarizeItem(item) {
  return {
    name: item.name ?? null,
    type: item.type ?? null,
    count: item.count ?? 1,
    displayName: item.displayName ?? null,
    nbt: item.nbt ? true : undefined,
    components: item.components ? Object.keys(item.components).sort() : undefined
  }
}

function delay(ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runEnchantmentSmoke({
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
