import { runObservedOfflineLogin } from './login_session.mjs'
import { waitForSpawn } from './bot_actions.mjs'

export const PLAY_BOUNDARY_RECONNECT_PHASES = [
  'join-game',
  'first-chunk',
  'first-physics-tick'
]

export function createPlayBoundaryReconnectPlan(options = {}) {
  return {
    name: 'mineflayer-play-boundary-reconnect',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'BoundaryBot',
    phases: PLAY_BOUNDARY_RECONNECT_PHASES,
    serverProperties: {
      'online-mode': 'false',
      gamemode: 'survival',
      difficulty: 'peaceful',
      'spawn-protection': '0'
    }
  }
}

export function summarizePlayBoundaryReconnect(evidence, plan) {
  const phases = Object.fromEntries(plan.phases.map(phase => [phase, false]))
  for (const event of evidence.timeline ?? []) {
    if (event.name === 'boundary_reconnect' && phases[event.summary?.[0]] !== undefined) {
      phases[event.summary[0]] = true
    }
  }
  return {
    ok: Object.values(phases).every(Boolean),
    phases,
    failures: (evidence.timeline ?? []).filter(event => ['kicked', 'error'].includes(event.name))
  }
}

export async function runPlayBoundaryReconnect(options = {}) {
  const plan = createPlayBoundaryReconnectPlan(options)
  const evidence = { timeline: [] }

  for (const phase of plan.phases) {
    const first = await connectForBoundaryPhase(plan, phase, options)
    try {
      await (options.waitForPhase ?? waitForPhase)(first, phase, options)
      first.bot?.end?.()
      recordBoundary(evidence, `${phase}:disconnect`, { uuid: first.uuid })
    } finally {
      await first.cleanup?.()
    }

    const retry = await connectForBoundaryPhase(plan, phase, options)
    try {
      await (options.verifyReconnectParity ?? verifyReconnectParity)(retry, plan, phase, options)
      recordBoundary(evidence, phase, {
        username: retry.profile?.username,
        uuid: retry.uuid
      })
    } finally {
      await retry.cleanup?.()
    }
  }

  return {
    plan,
    evidence,
    summary: summarizePlayBoundaryReconnect(evidence, plan)
  }
}

export function recordBoundary(evidence, phase, details = {}) {
  evidence.timeline.push({ name: 'boundary_reconnect', at: Date.now(), summary: [phase, details] })
  return { phase, ...details }
}

async function connectForBoundaryPhase(plan, phase, options) {
  const session = await (options.runObservedOfflineLogin ?? runObservedOfflineLogin)({
    ...options,
    username: `${plan.username}_${phase.replaceAll('-', '')}`,
    properties: {
      ...plan.serverProperties,
      ...(options.properties ?? {})
    },
    keepAlive: true
  })
  if (session.error) throw session.error
  return session
}

async function waitForPhase(session, phase, options = {}) {
  if (phase === 'join-game') {
    if (!session.timeline?.some(event => event.name === 'login')) await waitForSpawn(session, options)
    return
  }
  if (phase === 'first-chunk') {
    await waitForSpawn(session, options)
    const position = session.bot?.entity?.position
    const block = position ? session.bot.blockAt(position.offset(0, -1, 0)) : null
    if (!block) throw new Error('Expected chunk-visible block before first-chunk boundary disconnect')
    return
  }
  if (phase === 'first-physics-tick') {
    await waitForSpawn(session, options)
    await new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error('Timed out waiting for first physics tick')), options.timeoutMs ?? 30_000)
      session.bot.once('physicsTick', () => {
        clearTimeout(timer)
        resolve()
      })
    })
  }
}

async function verifyReconnectParity(session, plan, phase, options = {}) {
  await (options.waitForSpawn ?? waitForSpawn)(session, { timeoutMs: options.timeoutMs })
  if (session.profile?.username == null || !session.profile.username.startsWith(plan.username)) {
    throw new Error(`Reconnect after ${phase} returned wrong profile`)
  }
  const position = session.bot?.entity?.position
  if (!position) throw new Error(`Reconnect after ${phase} did not restore visible player entity`)
  return {
    username: session.profile.username,
    uuid: session.uuid,
    position: { x: position.x, y: position.y, z: position.z }
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runPlayBoundaryReconnect({
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
