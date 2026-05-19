import { offlineUuid } from './runner.mjs'

export const SPAWN_CHUNK_SCENARIOS = {
  firstLoginSpawnParity: [
    'initial-spawn-block',
    'yaw-pitch',
    'dimension',
    'world-seed',
    'spawn-protection',
    'official-server-comparison'
  ],
  spawnAreaSafety: [
    'multi-seed-worlds',
    'vanilla-spawn-search',
    'collision-free-placement',
    'immediate-chunk-availability'
  ],
  chunkStreaming: [
    'initial-chunks-ready',
    'view-distance-change',
    'simulation-distance-change',
    'chunk-boundary-loads',
    'chunk-unload-order'
  ],
  slowInitialChunk: [
    'first-chunk-delayed',
    'keepalive-survives-delay',
    'loading-state-visible',
    'eventual-spawn-readiness'
  ],
  chunkResend: [
    'same-chunk-reconnect',
    'long-distance-teleport',
    'dimension-teleport',
    'stale-chunks-unloaded-before-new-terrain'
  ],
  forcedChunkVisibility: [
    'forceload-command-applied',
    'move-away',
    'move-back',
    'forced-chunk-remains-available'
  ]
}

export function createSpawnChunkScenarioPlan(kind, options = {}) {
  const steps = SPAWN_CHUNK_SCENARIOS[kind]
  if (!steps) throw new Error(`Unknown spawn/chunk scenario ${kind}`)
  const username = options.username ?? defaultUsername(kind)
  return {
    name: `mineflayer-${kebab(kind)}`,
    kind,
    mode: 'offline',
    auth: 'offline',
    username,
    uuid: offlineUuid(username),
    steps,
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false',
      'spawn-protection': options.spawnProtection ?? '16',
      'view-distance': options.viewDistance ?? '10',
      'simulation-distance': options.simulationDistance ?? '10'
    }
  }
}

export async function runSpawnChunkScenario(kind, options = {}) {
  const plan = createSpawnChunkScenarioPlan(kind, options)
  const evidence = { timeline: [] }
  const probe = options.probe ?? spawnChunkProbe
  const result = await probe(plan, options)
  for (const step of plan.steps) {
    if (!result.steps?.[step]) throw new Error(`Missing spawn/chunk evidence for ${step}`)
    recordSpawnChunk(evidence, step, result.details?.[step] ?? {})
  }
  return {
    plan,
    evidence,
    summary: summarizeSpawnChunk(evidence, plan)
  }
}

export function summarizeSpawnChunk(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'spawn_chunk')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return {
    ok: Object.values(steps).every(Boolean),
    steps
  }
}

export function recordSpawnChunk(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'spawn_chunk', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

function defaultUsername(kind) {
  return {
    firstLoginSpawnParity: 'SpawnParityBot',
    spawnAreaSafety: 'SpawnSafeBot',
    chunkStreaming: 'ChunkStreamBot',
    slowInitialChunk: 'SlowChunkBot',
    chunkResend: 'ChunkResendBot',
    forcedChunkVisibility: 'ForceChunkBot'
  }[kind] ?? 'ChunkBot'
}

function kebab(value) {
  return value.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)
}

async function spawnChunkProbe() {
  throw new Error('spawnChunkProbe requires a scenario-specific server fixture')
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const kind = process.argv[2] ?? 'firstLoginSpawnParity'
  runSpawnChunkScenario(kind).then(result => {
    console.log(JSON.stringify({ plan: result.plan, summary: result.summary }, null, 2))
    process.exitCode = result.summary.ok ? 0 : 1
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
