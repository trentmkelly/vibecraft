import { offlineUuid } from './runner.mjs'
import { runObservedOfflineLogin } from './login_session.mjs'

export const IDENTITY_NORMALIZATION_STEPS = [
  'original-casing-preserved',
  'offline-uuid-uses-original-case',
  'stored-profile-name-preserved',
  'command-selector-case-matches',
  'log-output-preserves-name'
]

export function createIdentityNormalizationPlan(options = {}) {
  const base = options.baseName ?? 'NormCase'
  return {
    name: 'mineflayer-identity-normalization',
    mode: 'offline',
    auth: 'offline',
    usernames: options.usernames ?? [base, base.toLowerCase(), base.toUpperCase()],
    steps: IDENTITY_NORMALIZATION_STEPS,
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false'
    }
  }
}

export async function runIdentityNormalization(options = {}) {
  const plan = createIdentityNormalizationPlan(options)
  const evidence = { timeline: [] }
  const joins = []

  for (const username of plan.usernames) {
    const joined = await (options.loginProbe ?? loginProbe)(username, options)
    joins.push(joined)
    if (joined.profile?.username !== username) {
      throw new Error(`Expected original casing ${username}, got ${joined.profile?.username}`)
    }
    const expectedUuid = offlineUuid(username)
    const actualUuid = joined.profile?.actualUuid ?? joined.profile?.expectedUuid
    if (actualUuid !== expectedUuid) {
      throw new Error(`Expected case-sensitive offline UUID ${expectedUuid}, got ${actualUuid}`)
    }
  }

  recordNormalization(evidence, 'original-casing-preserved', {
    names: joins.map(join => join.profile.username)
  })
  recordNormalization(evidence, 'offline-uuid-uses-original-case', {
    uuids: joins.map(join => join.profile.actualUuid ?? join.profile.expectedUuid)
  })

  const stored = await (options.storedProfileProbe ?? storedProfileProbe)(plan, joins, options)
  recordNormalization(evidence, 'stored-profile-name-preserved', stored)

  const selector = await (options.selectorProbe ?? selectorProbe)(plan, joins, options)
  recordNormalization(evidence, 'command-selector-case-matches', selector)

  const logs = await (options.logProbe ?? logProbe)(plan, joins, options)
  recordNormalization(evidence, 'log-output-preserves-name', logs)

  return {
    plan,
    evidence,
    summary: summarizeIdentityNormalization(evidence, plan)
  }
}

export function summarizeIdentityNormalization(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'identity_normalization')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return {
    ok: Object.values(steps).every(Boolean),
    steps
  }
}

export function recordNormalization(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'identity_normalization', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

async function loginProbe(username, options = {}) {
  const session = await runObservedOfflineLogin({
    ...options,
    username,
    keepAlive: false
  })
  if (session.error) throw session.error
  return session
}

async function storedProfileProbe() {
  throw new Error('storedProfileProbe requires a scenario-specific fixture')
}

async function selectorProbe() {
  throw new Error('selectorProbe requires a scenario-specific fixture')
}

async function logProbe() {
  throw new Error('logProbe requires a scenario-specific fixture')
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runIdentityNormalization({
    binary: process.env.VIBECRAFT_BIN,
    port: Number(process.env.VIBECRAFT_PORT ?? 25565),
    version: process.env.MINEFLAYER_VERSION,
    timeoutMs: Number(process.env.VIBECRAFT_TIMEOUT_MS ?? 30_000)
  }).then(result => {
    console.log(JSON.stringify({ plan: result.plan, summary: result.summary }, null, 2))
    process.exitCode = result.summary.ok ? 0 : 1
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
