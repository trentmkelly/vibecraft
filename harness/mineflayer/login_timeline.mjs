import { once } from 'node:events'
import { runObservedOfflineLogin } from './login_session.mjs'

export const LOGIN_TIMELINE_MILESTONES = [
  'tcp-connect-start',
  'login',
  'spawn',
  'first-physics-tick'
]

export function createLoginTimelinePlan(options = {}) {
  return {
    name: 'mineflayer-login-to-play-timeline',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'TimelineBot',
    maxLoginToSpawnMs: options.maxLoginToSpawnMs ?? 10_000,
    maxSpawnToPhysicsMs: options.maxSpawnToPhysicsMs ?? 5_000,
    maxPacketGapMs: options.maxPacketGapMs ?? 5_000,
    milestones: LOGIN_TIMELINE_MILESTONES,
    serverProperties: {
      'online-mode': 'false',
      gamemode: 'survival',
      difficulty: 'peaceful'
    }
  }
}

export async function runLoginTimeline(options = {}) {
  const plan = createLoginTimelinePlan(options)
  const vibecraft = await observeLoginTimeline({
    ...options,
    plan,
    label: 'vibecraft',
    serverKind: options.serverKind ?? 'vibecraft'
  })
  const oracle = options.skipOracle === true
    ? null
    : await observeLoginTimeline({
      ...options,
      plan,
      label: 'official',
      serverKind: 'official',
      port: options.oraclePort ?? ((options.port ?? 25565) + 1),
      root: options.oracleRoot,
      binary: options.binary,
      jar: options.jar
    })

  const comparison = compareLoginTimelines(vibecraft.summary, oracle?.summary, plan)
  return { plan, vibecraft, oracle, comparison }
}

export async function observeLoginTimeline(options = {}) {
  const plan = options.plan ?? createLoginTimelinePlan(options)
  const timeline = [{ name: 'timeline', at: Date.now(), summary: ['tcp-connect-start', { label: options.label }] }]
  const session = await (options.runObservedOfflineLogin ?? runObservedOfflineLogin)({
    ...options,
    username: plan.username,
    properties: {
      ...plan.serverProperties,
      ...(options.properties ?? {})
    },
    timeline,
    keepAlive: true
  })
  if (session.error) throw session.error

  try {
    await (options.waitForFirstPhysicsTick ?? waitForFirstPhysicsTick)(session, options)
    session.timeline.push({ name: 'timeline', at: options.now?.() ?? Date.now(), summary: ['first-physics-tick', {}] })
    return {
      session,
      summary: summarizeLoginTimeline(session, plan)
    }
  } finally {
    await session.cleanup?.()
  }
}

export function summarizeLoginTimeline(session, plan = createLoginTimelinePlan()) {
  const events = session.timeline ?? []
  const packetTrace = session.packetTrace ?? []
  const milestoneTimes = {
    'tcp-connect-start': eventTime(events, 'timeline', 'tcp-connect-start'),
    login: eventTime(events, 'login'),
    spawn: eventTime(events, 'spawn'),
    'first-physics-tick': eventTime(events, 'timeline', 'first-physics-tick')
  }
  const packetGaps = packetTrace
    .slice(1)
    .map((packet, index) => packet.at - packetTrace[index].at)
    .filter(gap => Number.isFinite(gap))
  const failures = []

  for (const name of plan.milestones) {
    if (!Number.isFinite(milestoneTimes[name])) failures.push(`missing:${name}`)
  }
  if (!ordered(plan.milestones.map(name => milestoneTimes[name]))) failures.push('milestone-order')
  if (duration(milestoneTimes.login, milestoneTimes.spawn) > plan.maxLoginToSpawnMs) failures.push('login-to-spawn-timeout')
  if (duration(milestoneTimes.spawn, milestoneTimes['first-physics-tick']) > plan.maxSpawnToPhysicsMs) failures.push('spawn-to-physics-timeout')
  if (packetGaps.some(gap => gap > plan.maxPacketGapMs)) failures.push('packet-gap-timeout')

  return {
    ok: failures.length === 0,
    failures,
    milestoneTimes,
    durations: {
      loginToSpawnMs: duration(milestoneTimes.login, milestoneTimes.spawn),
      spawnToPhysicsMs: duration(milestoneTimes.spawn, milestoneTimes['first-physics-tick'])
    },
    packetGaps
  }
}

export function compareLoginTimelines(actual, oracle, plan = createLoginTimelinePlan()) {
  const failures = [...(actual?.failures ?? [])]
  if (!actual?.ok) failures.push('actual-not-ok')
  if (oracle) {
    if (!oracle.ok) failures.push('oracle-not-ok')
    if (actual.durations.loginToSpawnMs > oracle.durations.loginToSpawnMs * 2 + 250) {
      failures.push('login-to-spawn-slower-than-oracle')
    }
    if (actual.durations.spawnToPhysicsMs > oracle.durations.spawnToPhysicsMs * 2 + 250) {
      failures.push('spawn-to-physics-slower-than-oracle')
    }
  }

  return {
    ok: failures.length === 0,
    failures,
    actual,
    oracle
  }
}

async function waitForFirstPhysicsTick(session, options = {}) {
  await onceWithTimeout(session.bot, 'physicsTick', options.timeoutMs ?? 30_000)
}

function eventTime(events, name, summaryName = null) {
  const event = events.find(candidate => {
    if (candidate.name !== name) return false
    return summaryName == null || candidate.summary?.[0] === summaryName
  })
  return event?.at ?? NaN
}

function duration(start, end) {
  if (!Number.isFinite(start) || !Number.isFinite(end)) return Infinity
  return end - start
}

function ordered(values) {
  return values.every(Number.isFinite) && values.every((value, index) => index === 0 || value >= values[index - 1])
}

function onceWithTimeout(emitter, event, timeoutMs) {
  return Promise.race([
    once(emitter, event),
    new Promise((_, reject) => setTimeout(() => reject(new Error(`Timed out waiting for ${event}`)), timeoutMs))
  ])
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runLoginTimeline({
    binary: process.env.VIBECRAFT_BIN,
    jar: process.env.VANILLA_SERVER_JAR,
    port: Number(process.env.VIBECRAFT_PORT ?? 25565),
    oraclePort: Number(process.env.VANILLA_ORACLE_PORT ?? 25566),
    version: process.env.MINEFLAYER_VERSION,
    timeoutMs: Number(process.env.VIBECRAFT_TIMEOUT_MS ?? 30_000),
    skipOracle: process.env.VANILLA_SERVER_JAR == null,
    keepArtifacts: process.env.VIBECRAFT_KEEP_ARTIFACTS === '1'
  }).then(result => {
    console.log(JSON.stringify({ plan: result.plan, comparison: result.comparison }, null, 2))
    process.exitCode = result.comparison.ok ? 0 : 1
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
