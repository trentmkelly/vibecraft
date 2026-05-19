import { offlineUuid } from './runner.mjs'

export const USERCACHE_STEPS = [
  'generated-profile-cached',
  'restart-preserves-cache',
  'cached-name-and-uuid',
  'expires-on-vanilla-format',
  'lookup-side-effects'
]

export const USERCACHE_CORRUPTION_CASES = [
  'missing',
  'empty',
  'malformed',
  'stale',
  'duplicate'
]

export function createUsercachePlan(options = {}) {
  return {
    name: 'mineflayer-usercache',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'CacheBot',
    steps: USERCACHE_STEPS,
    expiresOnPattern: /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2} \+0000$/
  }
}

export function createUsercacheCorruptionPlan(options = {}) {
  return {
    name: 'mineflayer-usercache-corruption',
    mode: 'offline',
    auth: 'offline',
    cases: options.cases ?? USERCACHE_CORRUPTION_CASES,
    expiresOnPattern: /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2} \+0000$/
  }
}

export async function runUsercacheScenario(options = {}) {
  const plan = createUsercachePlan(options)
  const evidence = { timeline: [] }
  const first = await (options.joinProbe ?? joinProbe)(plan.username, 'first', options)
  verifyCacheEntry(first.cacheEntry, plan.username, plan.expiresOnPattern)
  recordUsercache(evidence, 'generated-profile-cached', first.cacheEntry)

  const restarted = await (options.restartProbe ?? restartProbe)(plan.username, options)
  verifyCacheEntry(restarted.cacheEntry, plan.username, plan.expiresOnPattern)
  recordUsercache(evidence, 'restart-preserves-cache', restarted.cacheEntry)
  recordUsercache(evidence, 'cached-name-and-uuid', {
    name: restarted.cacheEntry.name,
    uuid: restarted.cacheEntry.uuid
  })
  recordUsercache(evidence, 'expires-on-vanilla-format', {
    expiresOn: restarted.cacheEntry.expiresOn
  })

  const lookup = await (options.lookupProbe ?? lookupProbe)(plan.username, options)
  if (!lookup.sideEffectsObserved) throw new Error('Expected usercache lookup side effects')
  recordUsercache(evidence, 'lookup-side-effects', lookup)

  return {
    plan,
    evidence,
    summary: summarizeUsercache(evidence, plan)
  }
}

export async function runUsercacheCorruptionScenario(options = {}) {
  const plan = createUsercacheCorruptionPlan(options)
  const evidence = { timeline: [] }
  for (const corruptionCase of plan.cases) {
    const repaired = await (options.repairProbe ?? repairProbe)(corruptionCase, options)
    verifyCacheEntry(repaired.cacheEntry, repaired.username, plan.expiresOnPattern)
    if (repaired.staleUuidPresent) throw new Error(`${corruptionCase} repair left stale UUID behind`)
    recordUsercache(evidence, corruptionCase, repaired)
  }
  return {
    plan,
    evidence,
    summary: summarizeUsercacheCorruption(evidence, plan)
  }
}

export function summarizeUsercache(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'usercache')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return { ok: Object.values(steps).every(Boolean), steps }
}

export function summarizeUsercacheCorruption(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'usercache')
    .map(event => event.summary?.[0])
  const cases = Object.fromEntries(plan.cases.map(name => [name, actions.includes(name)]))
  return { ok: Object.values(cases).every(Boolean), cases }
}

export function recordUsercache(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'usercache', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

function verifyCacheEntry(entry, username, expiresOnPattern) {
  if (entry?.name !== username) throw new Error(`Expected usercache name ${username}, got ${entry?.name}`)
  const expectedUuid = offlineUuid(username)
  if (entry?.uuid !== expectedUuid) throw new Error(`Expected usercache UUID ${expectedUuid}, got ${entry?.uuid}`)
  if (!expiresOnPattern.test(entry?.expiresOn ?? '')) throw new Error(`Invalid usercache expiresOn ${entry?.expiresOn}`)
}

async function joinProbe() {
  throw new Error('joinProbe requires a scenario-specific fixture')
}

async function restartProbe() {
  throw new Error('restartProbe requires a scenario-specific fixture')
}

async function lookupProbe() {
  throw new Error('lookupProbe requires a scenario-specific fixture')
}

async function repairProbe() {
  throw new Error('repairProbe requires a scenario-specific fixture')
}
