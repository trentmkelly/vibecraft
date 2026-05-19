import { offlineUuid } from './runner.mjs'

export const INVENTORY_PARITY_SCENARIOS = {
  equipmentSync: [
    'armor-sync',
    'offhand-sync',
    'selected-hotbar-sync',
    'item-pickup-sync',
    'item-drop-sync',
    'respawn-retention-rules',
    'disconnect-reconnect-persistence'
  ],
  loginPersistence: [
    'give-items-before-save',
    'disconnect-before-explicit-save',
    'disconnect-after-explicit-save',
    'reconnect-restore-timing'
  ],
  correction: [
    'slot-click-desync-corrected',
    'carried-item-desync-corrected',
    'selected-hotbar-desync-corrected',
    'creative-action-desync-corrected',
    'drop-packet-desync-corrected'
  ],
  loginBaseline: [
    'fresh-empty-inventory',
    'selected-slot-zero',
    'empty-carried-item',
    'recipe-book-baseline',
    'cursor-state-baseline'
  ],
  windowLifecycle: [
    'player-inventory-open-click-close',
    'chest-open-click-close-reopen',
    'furnace-open-click-close-reopen',
    'crafting-table-open-click-close-reopen',
    'anvil-open-click-close-reopen',
    'merchant-open-click-close-reopen',
    'disconnect-mid-window-reconnect'
  ],
  craftingRecipeBook: [
    'recipes-unlocked',
    'craft-2x2-grid',
    'craft-3x3-grid',
    'workstation-opened',
    'recipe-sync-verified',
    'result-slots-match-vanilla'
  ],
  packetTraceValidation: [
    'click-packet-trace',
    'quick-move-packet-trace',
    'drag-splitting-packet-trace',
    'creative-action-packet-trace',
    'vanilla-trace-comparison'
  ]
}

export function createInventoryParityPlan(kind, options = {}) {
  const steps = INVENTORY_PARITY_SCENARIOS[kind]
  if (!steps) throw new Error(`Unknown inventory parity scenario ${kind}`)
  const username = options.username ?? defaultUsername(kind)
  return {
    name: `mineflayer-inventory-${kebab(kind)}`,
    kind,
    mode: 'offline',
    auth: 'offline',
    username,
    uuid: offlineUuid(username),
    steps,
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false',
      gamemode: options.gamemode ?? 'survival'
    }
  }
}

export async function runInventoryParityScenario(kind, options = {}) {
  const plan = createInventoryParityPlan(kind, options)
  const evidence = { timeline: [] }
  const result = await (options.probe ?? inventoryParityProbe)(plan, options)
  for (const step of plan.steps) {
    if (!result.steps?.[step]) throw new Error(`Missing inventory parity evidence for ${step}`)
    recordInventoryParity(evidence, step, result.details?.[step] ?? {})
  }
  return {
    plan,
    evidence,
    summary: summarizeInventoryParity(evidence, plan)
  }
}

export function summarizeInventoryParity(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'inventory_parity')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return {
    ok: Object.values(steps).every(Boolean),
    steps
  }
}

export function recordInventoryParity(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'inventory_parity', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

function defaultUsername(kind) {
  return {
    equipmentSync: 'EquipSyncBot',
    loginPersistence: 'InventoryPersistBot',
    correction: 'InventoryCorrectBot',
    loginBaseline: 'InventoryBaselineBot',
    windowLifecycle: 'WindowLifeBot',
    craftingRecipeBook: 'CraftRecipeBot',
    packetTraceValidation: 'InventoryTraceBot'
  }[kind] ?? 'InventoryBot'
}

function kebab(value) {
  return value.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)
}

async function inventoryParityProbe() {
  throw new Error('inventoryParityProbe requires a scenario-specific server fixture')
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const kind = process.argv[2] ?? 'equipmentSync'
  runInventoryParityScenario(kind).then(result => {
    console.log(JSON.stringify({ plan: result.plan, summary: result.summary }, null, 2))
    process.exitCode = result.summary.ok ? 0 : 1
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
