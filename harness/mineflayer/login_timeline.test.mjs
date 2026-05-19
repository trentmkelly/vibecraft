import test from 'node:test'
import assert from 'node:assert/strict'
import {
  compareLoginTimelines,
  createLoginTimelinePlan,
  observeLoginTimeline,
  summarizeLoginTimeline
} from './login_timeline.mjs'

test('summarizeLoginTimeline validates order, packet gaps, and thresholds', () => {
  const plan = createLoginTimelinePlan({
    maxLoginToSpawnMs: 1000,
    maxSpawnToPhysicsMs: 500,
    maxPacketGapMs: 250
  })
  const session = fakeTimelineSession({
    times: [1000, 1100, 1300, 1400],
    packetTimes: [1110, 1120, 1200]
  })
  const summary = summarizeLoginTimeline(session, plan)
  assert.equal(summary.ok, true)
  assert.equal(summary.durations.loginToSpawnMs, 200)
  assert.deepEqual(summary.packetGaps, [10, 80])

  const slow = summarizeLoginTimeline(fakeTimelineSession({
    times: [1000, 1100, 2300, 2400],
    packetTimes: [1110, 1600]
  }), plan)
  assert.equal(slow.ok, false)
  assert.ok(slow.failures.includes('login-to-spawn-timeout'))
  assert.ok(slow.failures.includes('packet-gap-timeout'))
})

test('compareLoginTimelines flags actual runs slower than oracle envelope', () => {
  const plan = createLoginTimelinePlan({ maxLoginToSpawnMs: 10_000, maxSpawnToPhysicsMs: 5_000 })
  const oracle = summarizeLoginTimeline(fakeTimelineSession({
    times: [0, 50, 100, 150],
    packetTimes: [60, 70]
  }), plan)
  const actual = summarizeLoginTimeline(fakeTimelineSession({
    times: [0, 50, 1000, 1800],
    packetTimes: [60, 70]
  }), plan)
  const comparison = compareLoginTimelines(actual, oracle, plan)
  assert.equal(comparison.ok, false)
  assert.ok(comparison.failures.includes('login-to-spawn-slower-than-oracle'))
  assert.ok(comparison.failures.includes('spawn-to-physics-slower-than-oracle'))
})

test('observeLoginTimeline records first physics milestone and summarizes injected session', async () => {
  let cleaned = false
  let startAt = Date.now()
  const observed = await observeLoginTimeline({
    label: 'rustcraft',
    runObservedOfflineLogin: async options => {
      const base = options.timeline[0].at
      startAt = base
      return {
        timeline: [
          ...(options.timeline ?? []),
          { name: 'login', at: base + 20, summary: [] },
          { name: 'spawn', at: base + 40, summary: [] }
        ],
        packetTrace: [{ name: 'login', at: base + 25 }, { name: 'position', at: base + 35 }],
        bot: {},
        cleanup: async () => { cleaned = true }
      }
    },
    waitForFirstPhysicsTick: async session => {
      const base = session.timeline[0].at
      session.timeline.push({ name: 'test', at: base + 50, summary: ['physicsTick-observed'] })
    },
    now: () => startAt + 60
  })

  assert.equal(observed.summary.ok, true)
  assert.equal(cleaned, true)
  assert.equal(observed.session.timeline.at(-1).summary[0], 'first-physics-tick')
})

function fakeTimelineSession({ times, packetTimes }) {
  const [connectAt, loginAt, spawnAt, physicsAt] = times
  return {
    timeline: [
      { name: 'timeline', at: connectAt, summary: ['tcp-connect-start', {}] },
      { name: 'login', at: loginAt, summary: [] },
      { name: 'spawn', at: spawnAt, summary: [] },
      { name: 'timeline', at: physicsAt, summary: ['first-physics-tick', {}] }
    ],
    packetTrace: packetTimes.map((at, index) => ({ name: `packet-${index}`, at }))
  }
}
