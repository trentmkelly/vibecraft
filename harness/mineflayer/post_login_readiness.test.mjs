import test from 'node:test'
import assert from 'node:assert/strict'
import {
  POST_LOGIN_READINESS_STEPS,
  createPostLoginReadinessPlan,
  recordReadiness,
  runPostLoginReadiness,
  summarizePostLoginReadiness
} from './post_login_readiness.mjs'

test('createPostLoginReadinessPlan covers first-physics usable-action milestones', () => {
  const plan = createPostLoginReadinessPlan()
  assert.equal(plan.name, 'mineflayer-post-login-readiness')
  assert.deepEqual(plan.steps, POST_LOGIN_READINESS_STEPS)
  assert.ok(plan.steps.includes('movement-immediate'))
  assert.ok(plan.steps.includes('chat-immediate'))
  assert.ok(plan.steps.includes('command-suggestions-immediate'))
  assert.ok(plan.steps.includes('inventory-window-id'))
  assert.ok(plan.steps.includes('chunk-visibility'))
})

test('summarizePostLoginReadiness requires every readiness step', () => {
  const plan = createPostLoginReadinessPlan()
  const session = { timeline: [] }
  for (const step of plan.steps) recordReadiness(session, step)
  assert.equal(summarizePostLoginReadiness(session, plan).ok, true)

  session.timeline.pop()
  const summary = summarizePostLoginReadiness(session, plan)
  assert.equal(summary.ok, false)
  assert.equal(summary.steps['chunk-visibility'], false)
})

test('runPostLoginReadiness records no-retry actions after first physics tick', async () => {
  let cleaned = false
  const result = await runPostLoginReadiness({
    port: 25565,
    runObservedOfflineLogin: async () => ({
      timeline: [],
      bot: {},
      cleanup: async () => { cleaned = true }
    }),
    waitForSpawn: async () => {},
    waitForFirstPhysicsTick: async session => recordReadiness(session, 'test.physicsTick.observed'),
    verifyMovement: async session => recordReadiness(session, 'test.movement.called'),
    verifyChat: async session => recordReadiness(session, 'test.chat.called'),
    verifyCommandSuggestions: async () => ['list'],
    verifyInventoryWindowId: () => 0,
    verifyChunkVisibility: () => ({ name: 'grass_block', position: { x: 0, y: 79, z: 0 } })
  })

  assert.equal(result.summary.ok, true)
  assert.equal(cleaned, true)
  const readinessActions = result.session.timeline
    .filter(event => event.name === 'readiness')
    .map(event => event.summary[0])
  assert.ok(readinessActions.indexOf('first-physics-tick') < readinessActions.indexOf('movement-immediate'))
  assert.ok(readinessActions.includes('command-suggestions-immediate'))
  assert.ok(readinessActions.includes('inventory-window-id'))
  assert.ok(readinessActions.includes('chunk-visibility'))
})
