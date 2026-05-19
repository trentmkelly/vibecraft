import { offlineUuid } from './runner.mjs'
import { runObservedOfflineLogin } from './login_session.mjs'
import { waitForSpawn } from './bot_actions.mjs'

export const SAME_NAME_REPLACEMENT_STEPS = [
  'first-session-active',
  'second-session-replaces-first',
  'first-session-kicked-before-second-visible',
  'tab-list-replaced',
  'playerdata-owned-by-offline-uuid'
]

export function createSameNameReplacementPlan(options = {}) {
  return {
    name: 'mineflayer-same-name-replacement',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'DuplicateBot',
    steps: SAME_NAME_REPLACEMENT_STEPS,
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false'
    }
  }
}

export async function runSameNameReplacement(options = {}) {
  const plan = createSameNameReplacementPlan(options)
  const evidence = { timeline: [] }
  const first = await (options.connectBot ?? runObservedOfflineLogin)({
    ...options,
    username: plan.username,
    properties: plan.serverProperties,
    keepAlive: true,
    keepBot: true
  })
  if (first.error) throw first.error
  await (options.waitForSpawn ?? waitForSpawn)(first, { timeoutMs: options.timeoutMs })
  recordReplacement(evidence, 'first-session-active', { uuid: first.uuid })

  const second = await (options.connectBot ?? runObservedOfflineLogin)({
    ...options,
    username: plan.username,
    root: first.root,
    port: first.endpoint?.port ?? options.port,
    startServer: () => first.server,
    keepAlive: true,
    keepBot: true
  })
  if (second.error) throw second.error
  await (options.waitForSpawn ?? waitForSpawn)(second, { timeoutMs: options.timeoutMs })
  recordReplacement(evidence, 'second-session-replaces-first', { uuid: second.uuid })

  const kick = await (options.verifyKickOrder ?? verifyKickOrder)(first, second, plan)
  recordReplacement(evidence, 'first-session-kicked-before-second-visible', kick)

  const tab = await (options.verifyTabReplacement ?? verifyTabReplacement)(first, second, plan)
  recordReplacement(evidence, 'tab-list-replaced', tab)

  const playerdata = await (options.verifyPlayerdataOwnership ?? verifyPlayerdataOwnership)(first, second, plan)
  recordReplacement(evidence, 'playerdata-owned-by-offline-uuid', playerdata)

  second.bot?.end?.()
  await first.cleanup?.()
  return {
    plan,
    evidence,
    summary: summarizeSameNameReplacement(evidence, plan)
  }
}

export function summarizeSameNameReplacement(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'same_name')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return {
    ok: Object.values(steps).every(Boolean),
    steps
  }
}

export function recordReplacement(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'same_name', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

function verifyKickOrder(first, second) {
  const firstEnded = first.timeline?.some(event => ['kicked', 'end'].includes(event.name)) ?? false
  if (!firstEnded) throw new Error('First same-name session was not kicked/closed')
  const secondSpawned = second.timeline?.some(event => event.name === 'spawn') ?? true
  if (!secondSpawned) throw new Error('Second same-name session did not reach spawn')
  return { firstEnded, secondSpawned }
}

function verifyTabReplacement(_first, second, plan) {
  const players = Object.keys(second.bot?.players ?? {})
  if (players.length > 0 && !players.includes(plan.username)) {
    throw new Error(`${plan.username} missing from replacement tab list`)
  }
  return { players }
}

function verifyPlayerdataOwnership(_first, second, plan) {
  const expectedUuid = offlineUuid(plan.username)
  const actualUuid = second.uuid ?? second.profile?.actualUuid ?? second.profile?.expectedUuid
  if (actualUuid !== expectedUuid) {
    throw new Error(`Expected replacement UUID ${expectedUuid}, got ${actualUuid}`)
  }
  return { uuid: actualUuid }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runSameNameReplacement({
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
