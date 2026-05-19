import { offlineUuid } from './runner.mjs'

export const PLAYERDATA_ROUNDTRIP_FIELDS = [
  'position',
  'rotation',
  'inventory',
  'selectedSlot',
  'health',
  'food',
  'xp',
  'gameMode',
  'recipeBook',
  'stats'
]

export function createPlayerdataRoundTripPlan(options = {}) {
  const username = options.username ?? 'PersistRoundTrip'
  return {
    name: 'mineflayer-playerdata-round-trip',
    mode: 'offline',
    auth: 'offline',
    username,
    uuid: offlineUuid(username),
    fields: PLAYERDATA_ROUNDTRIP_FIELDS,
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false',
      gamemode: options.gameMode ?? 'survival'
    },
    beforeDisconnect: {
      position: options.position ?? { x: 4.5, y: 81, z: -3.5 },
      rotation: options.rotation ?? { yaw: 90, pitch: 12.5 },
      inventory: options.inventory ?? [{ slot: 'hotbar.0', item: 'minecraft:stone', count: 32 }],
      selectedSlot: options.selectedSlot ?? 0,
      health: options.health ?? 13,
      food: options.food ?? { level: 17, saturation: 2.5 },
      xp: options.xp ?? { level: 3, progress: 0.25, total: 23 },
      gameMode: options.persistedGameMode ?? 'creative',
      recipeBook: options.recipeBook ?? { recipes: ['minecraft:oak_planks'], toBeDisplayed: ['minecraft:oak_planks'] },
      stats: options.stats ?? { 'minecraft:custom/minecraft:jump': 2 }
    }
  }
}

export async function runPlayerdataRoundTrip(options = {}) {
  const plan = createPlayerdataRoundTripPlan(options)
  const evidence = { timeline: [] }

  const changed = await (options.changeStateProbe ?? changeStateProbe)(plan, options)
  assertFieldSet('changed state before disconnect', changed, plan.fields)
  recordPlayerdataRoundTrip(evidence, 'state-changed-before-disconnect', changed)

  const saved = await (options.saveProbe ?? saveProbe)(plan, options)
  if (!saved.saved || saved.uuid !== plan.uuid) {
    throw new Error(`Expected UUID-bound playerdata save for ${plan.uuid}`)
  }
  assertFieldSet('saved playerdata', saved, plan.fields)
  recordPlayerdataRoundTrip(evidence, 'playerdata-saved-after-disconnect', saved)

  const reconnected = await (options.reconnectProbe ?? reconnectProbe)(plan, options)
  if (!reconnected.loadedBeforeSpawn) {
    throw new Error('Expected playerdata to load before first visible spawn packet')
  }
  assertFieldSet('reconnected state', reconnected, plan.fields)
  assertRoundTrip(plan.beforeDisconnect, reconnected.fields)
  recordPlayerdataRoundTrip(evidence, 'state-restored-after-reconnect', reconnected)

  return {
    plan,
    evidence,
    summary: summarizePlayerdataRoundTrip(evidence, plan)
  }
}

export function summarizePlayerdataRoundTrip(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'playerdata_roundtrip')
    .map(event => event.summary?.[0])
  const steps = {
    changed: actions.includes('state-changed-before-disconnect'),
    saved: actions.includes('playerdata-saved-after-disconnect'),
    restored: actions.includes('state-restored-after-reconnect')
  }
  return {
    ok: Object.values(steps).every(Boolean),
    steps,
    fields: Object.fromEntries(plan.fields.map(field => [field, true]))
  }
}

export function recordPlayerdataRoundTrip(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'playerdata_roundtrip', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

export function assertRoundTrip(expected, actual) {
  for (const field of PLAYERDATA_ROUNDTRIP_FIELDS) {
    const left = JSON.stringify(normalizeField(expected[field]))
    const right = JSON.stringify(normalizeField(actual[field]))
    if (left !== right) {
      throw new Error(`Playerdata ${field} mismatch: expected ${left}, got ${right}`)
    }
  }
}

function assertFieldSet(label, evidence, fields) {
  const found = evidence.fields ?? {}
  for (const field of fields) {
    if (!(field in found)) throw new Error(`Missing ${field} in ${label}`)
  }
}

function normalizeField(value) {
  if (Array.isArray(value)) {
    return value.map(normalizeField)
  }
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.entries(value)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([key, nested]) => [key, normalizeField(nested)]))
  }
  return value
}

async function changeStateProbe() {
  throw new Error('changeStateProbe requires a scenario-specific server fixture')
}

async function saveProbe() {
  throw new Error('saveProbe requires a scenario-specific server fixture')
}

async function reconnectProbe() {
  throw new Error('reconnectProbe requires a scenario-specific server fixture')
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runPlayerdataRoundTrip().then(result => {
    console.log(JSON.stringify({ plan: result.plan, summary: result.summary }, null, 2))
    process.exitCode = result.summary.ok ? 0 : 1
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
