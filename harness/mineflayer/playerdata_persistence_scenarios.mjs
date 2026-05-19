import { offlineUuid } from './runner.mjs'

export const PLAYERDATA_PERSISTENCE_SCENARIOS = {
  freshProfile: [
    'login-without-movement',
    'immediate-profile-cache-observed',
    'no-early-progress-sidecars',
    'first-save-playerdata-created'
  ],
  reconnectAfterSave: [
    'record-state-before-disconnect',
    'wait-for-save-completion',
    'reconnect-loads-before-visible-spawn'
  ],
  dirtySave: [
    'movement-inventory-damage-stat-dirty',
    'disconnect-immediately',
    'restart-loads-flushed-state'
  ],
  abruptDisconnect: [
    'dirty-state-before-socket-destroy',
    'socket-destroyed',
    'restart-loads-last-saved-state',
    'session-cleanup-complete'
  ],
  firstLoginFiles: [
    'no-early-playerdata',
    'playerdata-created-at-first-save',
    'stats-advancements-recipe-sidecars-match-vanilla',
    'last-known-position-sidecar-match'
  ],
  uuidOwnership: [
    'two-generated-profiles-saved',
    'swapped-file-not-reassigned',
    'missing-file-regenerates-own-profile',
    'other-profile-remains-owned'
  ],
  corruptionLogin: [
    'truncated-playerdata-fallback',
    'wrong-compression-fallback',
    'wrong-uuid-recovery',
    'wrong-dimension-recovery',
    'warnings-observed'
  ]
}

export function createPlayerdataPersistencePlan(kind, options = {}) {
  const steps = PLAYERDATA_PERSISTENCE_SCENARIOS[kind]
  if (!steps) throw new Error(`Unknown playerdata persistence scenario ${kind}`)
  const username = options.username ?? defaultUsername(kind)
  return {
    name: `mineflayer-playerdata-${kebab(kind)}`,
    kind,
    mode: 'offline',
    auth: 'offline',
    username,
    uuid: offlineUuid(username),
    steps,
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false'
    }
  }
}

export async function runPlayerdataPersistenceScenario(kind, options = {}) {
  const plan = createPlayerdataPersistencePlan(kind, options)
  const evidence = { timeline: [] }
  const probe = options.probe ?? persistenceProbe
  const result = await probe(plan, options)
  for (const step of plan.steps) {
    if (!result.steps?.[step]) throw new Error(`Missing playerdata persistence evidence for ${step}`)
    recordPlayerdataPersistence(evidence, step, result.details?.[step] ?? {})
  }
  return {
    plan,
    evidence,
    summary: summarizePlayerdataPersistence(evidence, plan)
  }
}

export function summarizePlayerdataPersistence(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'playerdata_persistence')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return {
    ok: Object.values(steps).every(Boolean),
    steps
  }
}

export function recordPlayerdataPersistence(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'playerdata_persistence', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

function defaultUsername(kind) {
  return {
    freshProfile: 'FreshProfileBot',
    reconnectAfterSave: 'ReconnectSavedBot',
    dirtySave: 'DirtySaveBot',
    abruptDisconnect: 'AbruptPersistBot',
    firstLoginFiles: 'FirstFilesBot',
    uuidOwnership: 'OwnerOne',
    corruptionLogin: 'CorruptLoginBot'
  }[kind] ?? 'PersistBot'
}

function kebab(value) {
  return value.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)
}

async function persistenceProbe() {
  throw new Error('persistenceProbe requires a scenario-specific server fixture')
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const kind = process.argv[2] ?? 'freshProfile'
  runPlayerdataPersistenceScenario(kind).then(result => {
    console.log(JSON.stringify({ plan: result.plan, summary: result.summary }, null, 2))
    process.exitCode = result.summary.ok ? 0 : 1
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
