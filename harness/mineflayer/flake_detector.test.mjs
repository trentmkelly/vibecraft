import test from 'node:test'
import assert from 'node:assert/strict'
import {
  analyzeFlakeRuns,
  formatFlakeReport,
  runLoginFlakeDetector
} from './flake_detector.mjs'

test('runLoginFlakeDetector repeats shard runs on randomized or supplied ports', async () => {
  const seenPorts = []
  const result = await runLoginFlakeDetector({
    iterations: 3,
    ports: [26001, 26002, 26003],
    runShard: async ({ port, iteration }) => {
      seenPorts.push(port)
      return shard({ ok: true, durationMs: 10 + iteration })
    }
  })
  assert.equal(result.ok, true)
  assert.deepEqual(seenPorts, [26001, 26002, 26003])
  assert.equal(result.report.timing.varianceMs, 2)
})

test('analyzeFlakeRuns reports failed runs, intermittent kicks, and leaked processes', () => {
  const report = analyzeFlakeRuns([
    { iteration: 0, ok: true, durationMs: 10, kicked: false, leakedProcess: false },
    { iteration: 1, ok: false, durationMs: 25, kicked: true, leakedProcess: false },
    { iteration: 2, ok: true, durationMs: 40, kicked: false, leakedProcess: true }
  ])
  assert.deepEqual(report.timing, { minMs: 10, maxMs: 40, averageMs: 25, varianceMs: 30 })
  assert.deepEqual(report.failedRuns.map(run => run.iteration), [1])
  assert.deepEqual(report.intermittentKicks.map(run => run.iteration), [1])
  assert.deepEqual(report.leakedProcesses.map(run => run.iteration), [2])
})

test('runLoginFlakeDetector detects kicked timelines and leaked child processes', async () => {
  const result = await runLoginFlakeDetector({
    iterations: 2,
    ports: [26004, 26005],
    runShard: async ({ iteration }) => shard({
      ok: iteration === 0,
      durationMs: 5,
      session: {
        timeline: iteration === 1 ? [{ name: 'kicked', summary: ['bye'] }] : [],
        server: { child: iteration === 1 ? { exitCode: null, signalCode: null } : { exitCode: 0, signalCode: null } }
      }
    })
  })
  assert.equal(result.ok, false)
  assert.deepEqual(result.report.failedRuns.map(run => run.iteration), [1])
  assert.deepEqual(result.report.intermittentKicks.map(run => run.iteration), [1])
  assert.deepEqual(result.report.leakedProcesses.map(run => run.iteration), [1])
})

test('formatFlakeReport emits stable summary lines', () => {
  const text = formatFlakeReport({
    iterations: 2,
    timing: { minMs: 10, maxMs: 20, averageMs: 15, varianceMs: 10 },
    failedRuns: [{ iteration: 1 }],
    intermittentKicks: [],
    leakedProcesses: [{ iteration: 0 }]
  })
  assert.equal(text, [
    'iterations: 2',
    'timing: min=10ms max=20ms avg=15ms variance=10ms',
    'failedRuns: 1',
    'intermittentKicks: <none>',
    'leakedProcesses: 0'
  ].join('\n'))
})

function shard(options) {
  return {
    ok: options.ok,
    durationMs: options.durationMs,
    gate: { session: options.session ?? { timeline: [], server: { child: { exitCode: 0, signalCode: null } } } },
    milestones: [],
    report: options.ok ? 'pass' : 'fail'
  }
}
