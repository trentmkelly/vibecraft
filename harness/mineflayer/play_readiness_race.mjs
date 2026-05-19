import { runFirstActionMatrix } from './first_action_matrix.mjs'

export const PLAY_READINESS_RACE_ACTION_SETS = [
  ['movement', 'chat', 'command-suggestion'],
  ['inventory-click', 'block-dig'],
  ['movement', 'command-suggestion', 'block-place']
]

export function createPlayReadinessRacePlan(options = {}) {
  return {
    name: 'mineflayer-play-readiness-race',
    mode: 'offline',
    auth: 'offline',
    iterations: options.iterations ?? PLAY_READINESS_RACE_ACTION_SETS.length,
    actionSets: options.actionSets ?? PLAY_READINESS_RACE_ACTION_SETS,
    randomizedChunkDelaysMs: options.randomizedChunkDelaysMs ?? [0, 25, 75],
    retrySleepAllowed: false
  }
}

export async function runPlayReadinessRace(options = {}) {
  const plan = createPlayReadinessRacePlan(options)
  const evidence = { timeline: [] }

  for (let index = 0; index < plan.iterations; index++) {
    const actions = plan.actionSets[index % plan.actionSets.length]
    const chunkDelayMs = plan.randomizedChunkDelaysMs[index % plan.randomizedChunkDelaysMs.length]
    const result = await (options.runFirstActionMatrix ?? runFirstActionMatrix)({
      ...options,
      username: `${options.username ?? 'RaceBot'}${index}`,
      chunkDelayMs,
      actionSet: actions
    })
    const summary = summarizeIteration(result, actions, chunkDelayMs)
    evidence.timeline.push({
      name: 'play_readiness_race',
      at: Date.now(),
      summary: [`iteration-${index}`, summary]
    })
    if (!summary.ok) {
      throw new Error(`Play readiness race failed at iteration ${index}: ${summary.failures.join(',')}`)
    }
  }

  return {
    plan,
    evidence,
    summary: summarizePlayReadinessRace(evidence, plan)
  }
}

export function summarizePlayReadinessRace(evidence, plan) {
  const iterations = (evidence.timeline ?? []).filter(event => event.name === 'play_readiness_race')
  return {
    ok: iterations.length === plan.iterations && iterations.every(event => event.summary?.[1]?.ok),
    iterations: iterations.map(event => event.summary?.[1]),
    retrySleepAllowed: plan.retrySleepAllowed
  }
}

function summarizeIteration(result, actions, chunkDelayMs) {
  const failures = []
  if (!result.summary?.ok) failures.push('first-action-matrix')
  if ((result.session?.timeline ?? []).some(event => event.summary?.[0] === 'retry-sleep')) {
    failures.push('retry-sleep-used')
  }
  for (const action of actions) {
    if (!result.session?.timeline?.some(event => event.summary?.[0] === action)) {
      failures.push(`missing:${action}`)
    }
  }
  return {
    ok: failures.length === 0,
    actions,
    chunkDelayMs,
    failures
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runPlayReadinessRace({
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
