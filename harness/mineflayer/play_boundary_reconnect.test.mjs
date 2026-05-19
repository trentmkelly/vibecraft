import test from 'node:test'
import assert from 'node:assert/strict'
import {
  PLAY_BOUNDARY_RECONNECT_PHASES,
  createPlayBoundaryReconnectPlan,
  recordBoundary,
  runPlayBoundaryReconnect,
  summarizePlayBoundaryReconnect
} from './play_boundary_reconnect.mjs'

test('createPlayBoundaryReconnectPlan covers join, first chunk, and first physics tick', () => {
  const plan = createPlayBoundaryReconnectPlan()
  assert.equal(plan.name, 'mineflayer-play-boundary-reconnect')
  assert.deepEqual(plan.phases, PLAY_BOUNDARY_RECONNECT_PHASES)
})

test('summarizePlayBoundaryReconnect requires every phase', () => {
  const plan = createPlayBoundaryReconnectPlan()
  const evidence = { timeline: [] }
  for (const phase of plan.phases) recordBoundary(evidence, phase)
  assert.equal(summarizePlayBoundaryReconnect(evidence, plan).ok, true)

  evidence.timeline = evidence.timeline.filter(event => event.summary[0] !== 'first-chunk')
  const summary = summarizePlayBoundaryReconnect(evidence, plan)
  assert.equal(summary.ok, false)
  assert.equal(summary.phases['first-chunk'], false)
})

test('runPlayBoundaryReconnect disconnects and retries each boundary phase', async () => {
  const phases = []
  const ended = []
  const result = await runPlayBoundaryReconnect({
    port: 25565,
    runObservedOfflineLogin: async options => ({
      profile: { username: options.username },
      uuid: `${options.username}-uuid`,
      timeline: [{ name: 'login', summary: [] }],
      bot: {
        entity: { position: { x: 0, y: 80, z: 0 } },
        end: () => ended.push(options.username)
      },
      cleanup: async () => {}
    }),
    waitForPhase: async (_session, phase) => phases.push(`wait:${phase}`),
    verifyReconnectParity: async (_session, _plan, phase) => phases.push(`verify:${phase}`)
  })

  assert.equal(result.summary.ok, true)
  assert.deepEqual(phases, [
    'wait:join-game', 'verify:join-game',
    'wait:first-chunk', 'verify:first-chunk',
    'wait:first-physics-tick', 'verify:first-physics-tick'
  ])
  assert.equal(ended.length, 3)
})
