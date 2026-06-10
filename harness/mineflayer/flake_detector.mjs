import { runMinimalLoginShard } from './ci_login_shard.mjs'

export async function runLoginFlakeDetector(options = {}) {
  const iterations = options.iterations ?? 5
  const runs = []
  for (let index = 0; index < iterations; index++) {
    const port = options.ports?.[index] ?? randomPort(options.portBase ?? 25_600, options.portSpan ?? 1_000)
    const startedAt = Date.now()
    const result = await (options.runShard ?? runMinimalLoginShard)({
      ...options,
      iteration: index,
      port
    })
    const durationMs = result.durationMs ?? Date.now() - startedAt
    runs.push(summarizeRun(index, port, durationMs, result))
  }
  const report = analyzeFlakeRuns(runs)
  return {
    ok: report.failedRuns.length === 0 && report.intermittentKicks.length === 0 && report.leakedProcesses.length === 0,
    runs,
    report,
    text: formatFlakeReport(report)
  }
}

export function analyzeFlakeRuns(runs) {
  const durations = runs.map(run => run.durationMs)
  return {
    iterations: runs.length,
    timing: {
      minMs: Math.min(...durations),
      maxMs: Math.max(...durations),
      averageMs: Math.round(durations.reduce((sum, value) => sum + value, 0) / Math.max(durations.length, 1)),
      varianceMs: Math.max(...durations) - Math.min(...durations)
    },
    failedRuns: runs.filter(run => !run.ok),
    intermittentKicks: runs.filter(run => run.kicked),
    leakedProcesses: runs.filter(run => run.leakedProcess)
  }
}

export function formatFlakeReport(report) {
  return [
    `iterations: ${report.iterations}`,
    `timing: min=${report.timing.minMs}ms max=${report.timing.maxMs}ms avg=${report.timing.averageMs}ms variance=${report.timing.varianceMs}ms`,
    `failedRuns: ${report.failedRuns.map(run => run.iteration).join(',') || '<none>'}`,
    `intermittentKicks: ${report.intermittentKicks.map(run => run.iteration).join(',') || '<none>'}`,
    `leakedProcesses: ${report.leakedProcesses.map(run => run.iteration).join(',') || '<none>'}`
  ].join('\n')
}

function summarizeRun(iteration, port, durationMs, result) {
  const session = result.gate?.session ?? result.session ?? {}
  const child = session.server?.child
  return {
    iteration,
    port,
    ok: result.ok,
    durationMs,
    kicked: (session.timeline ?? []).some(event => event.name === 'kicked'),
    leakedProcess: Boolean(child && child.exitCode == null && child.signalCode == null),
    milestones: result.milestones ?? [],
    report: result.report
  }
}

function randomPort(base, span) {
  return base + Math.floor(Math.random() * span)
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const result = await runLoginFlakeDetector({
    iterations: Number(process.env.VIBECRAFT_FLAKE_RUNS ?? 5),
    binary: process.env.VIBECRAFT_BIN,
    username: process.env.VIBECRAFT_BOT ?? 'VibeCraftFlake',
    version: process.env.MINEFLAYER_VERSION,
    timeoutMs: Number(process.env.VIBECRAFT_TIMEOUT_MS ?? 30_000)
  })
  if (result.ok) {
    console.log(result.text)
  } else {
    console.error(result.text)
    process.exitCode = 1
  }
}
