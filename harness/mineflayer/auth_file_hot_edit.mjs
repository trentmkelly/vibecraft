import { offlineUuid } from './runner.mjs'

export const AUTH_FILE_HOT_EDIT_STEPS = [
  'online-bots-before-edit',
  'ops-json-hot-edit-reload',
  'whitelist-json-hot-edit-reload',
  'banned-players-json-hot-edit-reload',
  'banned-ips-json-hot-edit-reload',
  'current-bots-observe-reload',
  'reconnecting-bots-observe-reload',
  'pardon-and-unlist-observed'
]

export function createAuthFileHotEditPlan(options = {}) {
  const profiles = {
    plain: options.plainUsername ?? 'HotPlain',
    op: options.opUsername ?? 'HotOp',
    listed: options.listedUsername ?? 'HotListed',
    banned: options.bannedUsername ?? 'HotBanned',
    ipBanned: options.ipBannedUsername ?? 'HotIpBanned'
  }
  return {
    name: 'mineflayer-auth-file-hot-edit',
    mode: 'offline',
    auth: 'offline',
    profiles,
    expectedUuids: Object.fromEntries(
      Object.entries(profiles).map(([key, name]) => [key, offlineUuid(name)])
    ),
    files: ['ops.json', 'whitelist.json', 'banned-players.json', 'banned-ips.json'],
    reloadCommands: ['reload', 'whitelist reload'],
    steps: AUTH_FILE_HOT_EDIT_STEPS,
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false',
      'enforce-whitelist': 'true'
    }
  }
}

export async function runAuthFileHotEdit(options = {}) {
  const plan = createAuthFileHotEditPlan(options)
  const evidence = { timeline: [] }

  const online = await (options.onlineProbe ?? onlineProbe)(plan, options)
  if (!online.joined?.includes(plan.profiles.plain) || !online.joined?.includes(plan.profiles.op)) {
    throw new Error('Expected baseline online bots before auth file edits')
  }
  recordAuthHotEdit(evidence, 'online-bots-before-edit', online)

  const ops = await (options.opsHotEditProbe ?? opsHotEditProbe)(plan, options)
  if (!ops.reloaded || !ops.currentOp || !ops.reconnectOp) {
    throw new Error('Expected ops.json hot edit to apply to current and reconnecting bots')
  }
  recordAuthHotEdit(evidence, 'ops-json-hot-edit-reload', ops)

  const whitelist = await (options.whitelistHotEditProbe ?? whitelistHotEditProbe)(plan, options)
  if (!whitelist.reloaded || !whitelist.allowed || !whitelist.unlistedRejected) {
    throw new Error('Expected whitelist.json hot edit to gate reconnecting bots after reload')
  }
  recordAuthHotEdit(evidence, 'whitelist-json-hot-edit-reload', whitelist)

  const playerBan = await (options.playerBanHotEditProbe ?? playerBanHotEditProbe)(plan, options)
  if (!playerBan.reloaded || !playerBan.reconnectRejected || !/disconnect\.banned|banned/i.test(playerBan.reason ?? '')) {
    throw new Error(`Expected banned-players.json hot edit rejection, got ${playerBan.reason}`)
  }
  recordAuthHotEdit(evidence, 'banned-players-json-hot-edit-reload', playerBan)

  const ipBan = await (options.ipBanHotEditProbe ?? ipBanHotEditProbe)(plan, options)
  if (!ipBan.reloaded || !ipBan.reconnectRejected || !/disconnect\.ip_banned|ip.*banned/i.test(ipBan.reason ?? '')) {
    throw new Error(`Expected banned-ips.json hot edit rejection, got ${ipBan.reason}`)
  }
  recordAuthHotEdit(evidence, 'banned-ips-json-hot-edit-reload', ipBan)

  const current = await (options.currentBotProbe ?? currentBotProbe)(plan, options)
  if (!current.observedReload) throw new Error('Expected already-online bots to observe reload effects')
  recordAuthHotEdit(evidence, 'current-bots-observe-reload', current)

  const reconnecting = await (options.reconnectProbe ?? reconnectProbe)(plan, options)
  if (!reconnecting.observedReload) throw new Error('Expected reconnecting bots to observe reload effects')
  recordAuthHotEdit(evidence, 'reconnecting-bots-observe-reload', reconnecting)

  const pardon = await (options.pardonProbe ?? pardonProbe)(plan, options)
  if (!pardon.reloaded || !pardon.playerBanCleared || !pardon.ipBanCleared || !pardon.whitelistCleared) {
    throw new Error('Expected pardon/unlist hot edits to apply after reload')
  }
  recordAuthHotEdit(evidence, 'pardon-and-unlist-observed', pardon)

  return {
    plan,
    evidence,
    summary: summarizeAuthFileHotEdit(evidence, plan)
  }
}

export function summarizeAuthFileHotEdit(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'auth_file_hot_edit')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return {
    ok: Object.values(steps).every(Boolean),
    steps
  }
}

export function recordAuthHotEdit(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'auth_file_hot_edit', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

async function onlineProbe() {
  throw new Error('onlineProbe requires a scenario-specific server fixture')
}

async function opsHotEditProbe() {
  throw new Error('opsHotEditProbe requires a scenario-specific server fixture')
}

async function whitelistHotEditProbe() {
  throw new Error('whitelistHotEditProbe requires a scenario-specific server fixture')
}

async function playerBanHotEditProbe() {
  throw new Error('playerBanHotEditProbe requires a scenario-specific server fixture')
}

async function ipBanHotEditProbe() {
  throw new Error('ipBanHotEditProbe requires a scenario-specific server fixture')
}

async function currentBotProbe() {
  throw new Error('currentBotProbe requires a scenario-specific server fixture')
}

async function reconnectProbe() {
  throw new Error('reconnectProbe requires a scenario-specific server fixture')
}

async function pardonProbe() {
  throw new Error('pardonProbe requires a scenario-specific server fixture')
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runAuthFileHotEdit().then(result => {
    console.log(JSON.stringify({ plan: result.plan, summary: result.summary }, null, 2))
    process.exitCode = result.summary.ok ? 0 : 1
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
