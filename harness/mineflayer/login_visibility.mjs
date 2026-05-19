import { runObservedOfflineLogin } from './login_session.mjs'
import { waitForSpawn } from './bot_actions.mjs'

export const LOGIN_VISIBILITY_STEPS = [
  'not-visible-before-play',
  'tab-list-after-play',
  'nearby-player-spawned',
  'selector-visible-after-play'
]

export function createLoginVisibilityPlan(options = {}) {
  return {
    name: 'mineflayer-login-visibility',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'VisibleBot',
    observerUsername: options.observerUsername ?? 'VisibleObserver',
    steps: LOGIN_VISIBILITY_STEPS,
    serverProperties: {
      'online-mode': 'false',
      gamemode: 'survival',
      difficulty: 'peaceful',
      'spawn-protection': '0'
    }
  }
}

export function summarizeLoginVisibility(session, plan) {
  const actions = (session.timeline ?? [])
    .filter(event => event.name === 'visibility')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return {
    ok: Object.values(steps).every(Boolean),
    steps,
    failures: (session.timeline ?? []).filter(event => ['kicked', 'error'].includes(event.name))
  }
}

export async function runLoginVisibility(options = {}) {
  const plan = createLoginVisibilityPlan(options)
  const observer = await (options.runObservedOfflineLogin ?? runObservedOfflineLogin)({
    ...options,
    username: plan.observerUsername,
    properties: {
      ...plan.serverProperties,
      ...(options.properties ?? {})
    },
    keepAlive: true
  })
  if (observer.error) throw observer.error

  let joining
  try {
    await (options.waitForSpawn ?? waitForSpawn)(observer, { timeoutMs: options.timeoutMs })
    const before = await (options.capturePrePlayVisibility ?? capturePrePlayVisibility)(observer, plan, options)
    recordVisibility(observer, 'not-visible-before-play', before)

    joining = await (options.connectJoiningBot ?? runObservedOfflineLogin)({
      ...options,
      username: plan.username,
      root: observer.root,
      port: observer.endpoint?.port ?? options.port,
      startServer: () => observer.server,
      keepAlive: true,
      keepBot: true
    })
    if (joining.error) throw joining.error
    await (options.waitForSpawn ?? waitForSpawn)(joining, { timeoutMs: options.timeoutMs })

    const tab = await (options.verifyTabList ?? verifyTabList)(observer, joining, plan)
    recordVisibility(observer, 'tab-list-after-play', tab)

    const spawned = await (options.verifyNearbySpawn ?? verifyNearbySpawn)(observer, joining, plan)
    recordVisibility(observer, 'nearby-player-spawned', spawned)

    const selector = await (options.verifySelectorVisible ?? verifySelectorVisible)(observer, joining, plan)
    recordVisibility(observer, 'selector-visible-after-play', selector)

    return {
      plan,
      observer,
      joining,
      summary: summarizeLoginVisibility(observer, plan)
    }
  } finally {
    joining?.bot?.end?.()
    await observer.cleanup?.()
  }
}

export function recordVisibility(session, step, details = {}) {
  session.timeline.push({ name: 'visibility', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

function capturePrePlayVisibility(observer, plan) {
  const players = Object.keys(observer.bot?.players ?? {})
  if (players.includes(plan.username)) {
    throw new Error(`${plan.username} was visible before play-state join`)
  }
  return { players }
}

function verifyTabList(observer, joining, plan) {
  const players = Object.keys(observer.bot?.players ?? {})
  const visible = players.includes(plan.username) || players.includes(joining.profile?.username)
  if (!visible) throw new Error(`${plan.username} missing from observer tab list`)
  return { players }
}

function verifyNearbySpawn(observer, joining, plan) {
  const entity = Object.values(observer.bot?.entities ?? {}).find(candidate => {
    return candidate.username === plan.username || candidate.username === joining.profile?.username
  })
  if (!entity) throw new Error(`${plan.username} missing from observer entity list`)
  return {
    id: entity.id ?? null,
    username: entity.username ?? null,
    position: entity.position ? { x: entity.position.x, y: entity.position.y, z: entity.position.z } : null
  }
}

async function verifySelectorVisible(observer, joining, plan) {
  const query = `/execute as ${plan.username} run list`
  const response = typeof observer.bot?.tabComplete === 'function'
    ? await observer.bot.tabComplete(`/tell ${plan.username} `)
    : []
  const visible = response.length > 0 || Object.keys(observer.bot?.players ?? {}).includes(plan.username)
  if (!visible) throw new Error(`${plan.username} missing from command selector/suggestion visibility`)
  return { query, response }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runLoginVisibility({
    binary: process.env.RUSTCRAFT_BIN,
    port: Number(process.env.RUSTCRAFT_PORT ?? 25565),
    version: process.env.MINEFLAYER_VERSION,
    timeoutMs: Number(process.env.RUSTCRAFT_TIMEOUT_MS ?? 30_000),
    keepArtifacts: process.env.RUSTCRAFT_KEEP_ARTIFACTS === '1'
  }).then(result => {
    console.log(JSON.stringify({ plan: result.plan, summary: result.summary }, null, 2))
    process.exitCode = result.summary.ok ? 0 : 1
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
