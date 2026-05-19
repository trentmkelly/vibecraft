import { offlineUuid } from './runner.mjs'
import { runObservedOfflineLogin } from './login_session.mjs'

export const PROFILE_COLLISION_STEPS = [
  'case-variant-uuids-distinct',
  'display-names-preserved',
  'duplicate-session-handled',
  'playerdata-files-distinct'
]

export function createProfileCollisionPlan(options = {}) {
  const base = options.baseName ?? 'CaseProfile'
  return {
    name: 'mineflayer-profile-collision',
    mode: 'offline',
    auth: 'offline',
    usernames: options.usernames ?? [base, base.toLowerCase()],
    steps: PROFILE_COLLISION_STEPS,
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false'
    }
  }
}

export async function runProfileCollision(options = {}) {
  const plan = createProfileCollisionPlan(options)
  const evidence = { timeline: [] }
  const joins = []
  for (const username of plan.usernames) {
    const joined = await (options.loginProbe ?? loginProbe)(username, options)
    joins.push(joined)
  }

  const uuids = joins.map(join => join.profile?.actualUuid ?? join.profile?.expectedUuid)
  for (const [index, username] of plan.usernames.entries()) {
    const expectedUuid = offlineUuid(username)
    if (uuids[index] !== expectedUuid) throw new Error(`Expected UUID ${expectedUuid} for ${username}, got ${uuids[index]}`)
  }
  if (new Set(uuids).size !== uuids.length) throw new Error('Case-variant profiles collided on UUID')
  recordCollision(evidence, 'case-variant-uuids-distinct', { uuids })

  const names = joins.map(join => join.profile?.username)
  if (names.join('\0') !== plan.usernames.join('\0')) throw new Error('Display names were not preserved')
  recordCollision(evidence, 'display-names-preserved', { names })

  const duplicate = await (options.duplicateProbe ?? duplicateProbe)(plan, joins, options)
  recordCollision(evidence, 'duplicate-session-handled', duplicate)

  const playerdata = await (options.playerdataProbe ?? playerdataProbe)(plan, joins, options)
  recordCollision(evidence, 'playerdata-files-distinct', playerdata)

  return {
    plan,
    evidence,
    summary: summarizeProfileCollision(evidence, plan)
  }
}

export function summarizeProfileCollision(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'profile_collision')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return {
    ok: Object.values(steps).every(Boolean),
    steps
  }
}

export function recordCollision(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'profile_collision', at: Date.now(), summary: [step, details] })
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

async function duplicateProbe() {
  throw new Error('duplicateProbe requires a scenario-specific fixture')
}

async function playerdataProbe() {
  throw new Error('playerdataProbe requires a scenario-specific fixture')
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runProfileCollision({
    binary: process.env.RUSTCRAFT_BIN,
    port: Number(process.env.RUSTCRAFT_PORT ?? 25565),
    version: process.env.MINEFLAYER_VERSION,
    timeoutMs: Number(process.env.RUSTCRAFT_TIMEOUT_MS ?? 30_000)
  }).then(result => {
    console.log(JSON.stringify({ plan: result.plan, summary: result.summary }, null, 2))
    process.exitCode = result.summary.ok ? 0 : 1
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
