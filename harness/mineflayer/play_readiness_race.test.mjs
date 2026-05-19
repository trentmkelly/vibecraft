import test from 'node:test'
import assert from 'node:assert/strict'
import {
  PLAY_READINESS_RACE_ACTION_SETS,
  createPlayReadinessRacePlan,
  runPlayReadinessRace,
  summarizePlayReadinessRace
} from './play_readiness_race.mjs'

test('createPlayReadinessRacePlan repeats immediate action sets without retry sleeps', () => {
  const plan = createPlayReadinessRacePlan()
  assert.equal(plan.name, 'mineflayer-play-readiness-race')
  assert.deepEqual(plan.actionSets, PLAY_READINESS_RACE_ACTION_SETS)
  assert.equal(plan.retrySleepAllowed, false)
  assert.ok(plan.randomizedChunkDelaysMs.length > 1)
})

test('summarizePlayReadinessRace requires every iteration to pass', () => {
  const plan = createPlayReadinessRacePlan({ iterations: 2 })
  const evidence = {
    timeline: [
      { name: 'play_readiness_race', summary: ['iteration-0', { ok: true }] },
      { name: 'play_readiness_race', summary: ['iteration-1', { ok: true }] }
    ]
  }
  assert.equal(summarizePlayReadinessRace(evidence, plan).ok, true)
  evidence.timeline[1].summary[1].ok = false
  assert.equal(summarizePlayReadinessRace(evidence, plan).ok, false)
})

test('runPlayReadinessRace executes configured action sets and chunk delays', async () => {
  const calls = []
  const result = await runPlayReadinessRace({
    iterations: 3,
    actionSets: [['movement'], ['chat'], ['command-suggestion']],
    randomizedChunkDelaysMs: [0, 10, 20],
    runFirstActionMatrix: async options => {
      calls.push({ actions: options.actionSet, chunkDelayMs: options.chunkDelayMs })
      return {
        summary: { ok: true },
        session: {
          timeline: options.actionSet.map(action => ({ name: 'first_action', summary: [action, {}] }))
        }
      }
    }
  })

  assert.equal(result.summary.ok, true)
  assert.deepEqual(calls, [
    { actions: ['movement'], chunkDelayMs: 0 },
    { actions: ['chat'], chunkDelayMs: 10 },
    { actions: ['command-suggestion'], chunkDelayMs: 20 }
  ])
})

test('runPlayReadinessRace fails when an iteration uses retry sleep', async () => {
  await assert.rejects(() => runPlayReadinessRace({
    iterations: 1,
    actionSets: [['movement']],
    runFirstActionMatrix: async () => ({
      summary: { ok: true },
      session: {
        timeline: [
          { name: 'first_action', summary: ['movement', {}] },
          { name: 'first_action', summary: ['retry-sleep', {}] }
        ]
      }
    })
  }), /retry-sleep-used/)
})
