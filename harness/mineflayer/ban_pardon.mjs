import { offlineUuid } from './runner.mjs'

export const BAN_PARDON_STEPS = [
  'profile-ban-rejected',
  'ip-ban-rejected',
  'profile-pardon-reconnects',
  'ip-pardon-reconnects',
  'vanilla-compatible-messages'
]

export function createBanPardonPlan(options = {}) {
  return {
    name: 'mineflayer-ban-pardon',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'BanPardonBot',
    steps: BAN_PARDON_STEPS,
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false'
    }
  }
}

export async function runBanPardon(options = {}) {
  const plan = createBanPardonPlan(options)
  const evidence = { timeline: [] }

  const profileBan = await (options.profileBanProbe ?? profileBanProbe)(plan.username, options)
  assertRejected(profileBan, /multiplayer\.disconnect\.banned|banned/i, 'profile ban')
  recordBanPardon(evidence, 'profile-ban-rejected', profileBan)

  const ipBan = await (options.ipBanProbe ?? ipBanProbe)(plan.username, options)
  assertRejected(ipBan, /multiplayer\.disconnect\.ip_banned|ip.*banned/i, 'IP ban')
  recordBanPardon(evidence, 'ip-ban-rejected', ipBan)

  const profilePardon = await (options.profilePardonProbe ?? profilePardonProbe)(plan.username, options)
  assertReconnected(profilePardon, plan.username, 'profile pardon')
  recordBanPardon(evidence, 'profile-pardon-reconnects', profilePardon)

  const ipPardon = await (options.ipPardonProbe ?? ipPardonProbe)(plan.username, options)
  assertReconnected(ipPardon, plan.username, 'IP pardon')
  recordBanPardon(evidence, 'ip-pardon-reconnects', ipPardon)

  recordBanPardon(evidence, 'vanilla-compatible-messages', {
    profileBan: profileBan.reason,
    ipBan: ipBan.reason
  })

  return {
    plan,
    evidence,
    summary: summarizeBanPardon(evidence, plan)
  }
}

export function summarizeBanPardon(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'ban_pardon')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return {
    ok: Object.values(steps).every(Boolean),
    steps
  }
}

export function recordBanPardon(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'ban_pardon', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

function assertRejected(result, messagePattern, label) {
  if (!result.rejected) throw new Error(`Expected ${label} rejection`)
  if (!messagePattern.test(result.reason ?? '')) throw new Error(`Expected vanilla-compatible ${label} message, got ${result.reason}`)
}

function assertReconnected(result, username, label) {
  if (!result.ok) throw new Error(`Expected reconnect after ${label}`)
  const expectedUuid = offlineUuid(username)
  const actualUuid = result.profile?.actualUuid ?? result.profile?.expectedUuid ?? result.uuid
  if (actualUuid !== expectedUuid) throw new Error(`Expected ${label} UUID ${expectedUuid}, got ${actualUuid}`)
}

async function profileBanProbe() {
  throw new Error('profileBanProbe requires a scenario-specific fixture')
}

async function ipBanProbe() {
  throw new Error('ipBanProbe requires a scenario-specific fixture')
}

async function profilePardonProbe() {
  throw new Error('profilePardonProbe requires a scenario-specific fixture')
}

async function ipPardonProbe() {
  throw new Error('ipPardonProbe requires a scenario-specific fixture')
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runBanPardon().then(result => {
    console.log(JSON.stringify({ plan: result.plan, summary: result.summary }, null, 2))
    process.exitCode = result.summary.ok ? 0 : 1
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
