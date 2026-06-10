import { offlineUuid } from './runner.mjs'
import { runObservedOfflineLogin } from './login_session.mjs'

export const OFFLINE_IDENTITY_ACCESS_STEPS = [
  'username-preserved',
  'offline-uuid-derived',
  'whitelist-rejects-unlisted',
  'ban-rejects-profile',
  'operator-lookup-visible'
]

export function createOfflineIdentityAccessPlan(options = {}) {
  return {
    name: 'mineflayer-offline-identity-access',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'IdentityBot',
    steps: OFFLINE_IDENTITY_ACCESS_STEPS,
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false'
    }
  }
}

export async function runOfflineIdentityAccess(options = {}) {
  const plan = createOfflineIdentityAccessPlan(options)
  const evidence = { timeline: [] }

  const joined = await (options.loginProbe ?? loginProbe)(plan.username, {
    ...options,
    properties: plan.serverProperties
  })
  assertUsernameAndUuid(joined, plan.username)
  recordIdentity(evidence, 'username-preserved', { username: joined.profile.username })
  recordIdentity(evidence, 'offline-uuid-derived', { uuid: joined.profile.actualUuid })

  const whitelist = await (options.whitelistProbe ?? whitelistProbe)(plan.username, options)
  if (!whitelist.rejected) throw new Error('Expected unlisted whitelist login rejection')
  recordIdentity(evidence, 'whitelist-rejects-unlisted', whitelist)

  const ban = await (options.banProbe ?? banProbe)(plan.username, options)
  if (!ban.rejected) throw new Error('Expected banned profile login rejection')
  recordIdentity(evidence, 'ban-rejects-profile', ban)

  const op = await (options.operatorProbe ?? operatorProbe)(plan.username, options)
  if (!op.operatorVisible) throw new Error('Expected operator lookup evidence')
  recordIdentity(evidence, 'operator-lookup-visible', op)

  return {
    plan,
    evidence,
    summary: summarizeOfflineIdentityAccess(evidence, plan)
  }
}

export function summarizeOfflineIdentityAccess(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'identity')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return {
    ok: Object.values(steps).every(Boolean),
    steps
  }
}

export function recordIdentity(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'identity', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

function assertUsernameAndUuid(joined, username) {
  if (joined.profile?.username !== username) {
    throw new Error(`Expected username ${username}, got ${joined.profile?.username}`)
  }
  const expectedUuid = offlineUuid(username)
  if (joined.profile?.actualUuid !== expectedUuid && joined.profile?.expectedUuid !== expectedUuid) {
    throw new Error(`Expected offline UUID ${expectedUuid}, got ${joined.profile?.actualUuid ?? joined.profile?.expectedUuid}`)
  }
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

async function whitelistProbe() {
  throw new Error('whitelistProbe requires a scenario-specific server fixture')
}

async function banProbe() {
  throw new Error('banProbe requires a scenario-specific server fixture')
}

async function operatorProbe() {
  throw new Error('operatorProbe requires a scenario-specific server fixture')
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runOfflineIdentityAccess({
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
