import test from 'node:test'
import assert from 'node:assert/strict'
import {
  LOGIN_VISIBILITY_STEPS,
  createLoginVisibilityPlan,
  recordVisibility,
  runLoginVisibility,
  summarizeLoginVisibility
} from './login_visibility.mjs'

test('createLoginVisibilityPlan covers tab, entity, and selector visibility', () => {
  const plan = createLoginVisibilityPlan()
  assert.equal(plan.name, 'mineflayer-login-visibility')
  assert.deepEqual(plan.steps, LOGIN_VISIBILITY_STEPS)
  assert.ok(plan.steps.includes('not-visible-before-play'))
  assert.ok(plan.steps.includes('tab-list-after-play'))
  assert.ok(plan.steps.includes('nearby-player-spawned'))
  assert.ok(plan.steps.includes('selector-visible-after-play'))
})

test('summarizeLoginVisibility requires all visibility phases', () => {
  const plan = createLoginVisibilityPlan()
  const session = { timeline: [] }
  for (const step of plan.steps) recordVisibility(session, step)
  assert.equal(summarizeLoginVisibility(session, plan).ok, true)

  session.timeline = session.timeline.filter(event => event.summary[0] !== 'nearby-player-spawned')
  const summary = summarizeLoginVisibility(session, plan)
  assert.equal(summary.ok, false)
  assert.equal(summary.steps['nearby-player-spawned'], false)
})

test('runLoginVisibility observes hidden-before-play then visible-after-play evidence', async () => {
  let cleaned = false
  let joiningEnded = false
  const observer = {
    root: '/tmp/world',
    endpoint: { port: 25565 },
    server: { child: {} },
    timeline: [],
    bot: {
      players: { VisibleObserver: {}, VisibleBot: {} },
      entities: { 2: { id: 2, username: 'VisibleBot', position: { x: 0, y: 80, z: 0 } } },
      tabComplete: async () => ['VisibleBot']
    },
    cleanup: async () => { cleaned = true }
  }
  const joining = {
    profile: { username: 'VisibleBot' },
    bot: { end: () => { joiningEnded = true } }
  }
  const calls = []
  const result = await runLoginVisibility({
    port: 25565,
    runObservedOfflineLogin: async () => observer,
    waitForSpawn: async () => calls.push('spawn'),
    capturePrePlayVisibility: async () => ({ players: ['VisibleObserver'] }),
    connectJoiningBot: async () => joining
  })

  assert.equal(result.summary.ok, true)
  assert.equal(cleaned, true)
  assert.equal(joiningEnded, true)
  assert.deepEqual(calls, ['spawn', 'spawn'])
})

test('runLoginVisibility fails closed when pre-play visibility leaks', async () => {
  await assert.rejects(() => runLoginVisibility({
    port: 25565,
    runObservedOfflineLogin: async () => ({
      timeline: [],
      bot: {},
      cleanup: async () => {}
    }),
    waitForSpawn: async () => {},
    capturePrePlayVisibility: async () => {
      throw new Error('VisibleBot was visible before play-state join')
    }
  }), /visible before play-state/)
})
