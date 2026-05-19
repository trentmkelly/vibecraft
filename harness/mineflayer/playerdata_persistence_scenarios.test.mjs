import test from 'node:test'
import assert from 'node:assert/strict'
import {
  PLAYERDATA_PERSISTENCE_SCENARIOS,
  createPlayerdataPersistencePlan,
  recordPlayerdataPersistence,
  runPlayerdataPersistenceScenario,
  summarizePlayerdataPersistence
} from './playerdata_persistence_scenarios.mjs'
import { offlineUuid } from './runner.mjs'

for (const kind of Object.keys(PLAYERDATA_PERSISTENCE_SCENARIOS)) {
  test(`createPlayerdataPersistencePlan covers ${kind}`, () => {
    const plan = createPlayerdataPersistencePlan(kind)
    assert.equal(plan.name, `mineflayer-playerdata-${kind.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)}`)
    assert.equal(plan.uuid, offlineUuid(plan.username))
    assert.deepEqual(plan.steps, PLAYERDATA_PERSISTENCE_SCENARIOS[kind])
  })

  test(`runPlayerdataPersistenceScenario validates all ${kind} steps`, async () => {
    const result = await runPlayerdataPersistenceScenario(kind, {
      probe: async plan => ({
        steps: Object.fromEntries(plan.steps.map(step => [step, true])),
        details: Object.fromEntries(plan.steps.map(step => [step, { vanillaCompared: true }]))
      })
    })
    assert.equal(result.summary.ok, true)
  })
}

test('summarizePlayerdataPersistence fails when evidence is incomplete', () => {
  const plan = createPlayerdataPersistencePlan('dirtySave')
  const evidence = { timeline: [] }
  recordPlayerdataPersistence(evidence, plan.steps[0])
  assert.equal(summarizePlayerdataPersistence(evidence, plan).ok, false)
})

test('runPlayerdataPersistenceScenario reports the missing scenario step', async () => {
  await assert.rejects(() => runPlayerdataPersistenceScenario('corruptionLogin', {
    probe: async plan => ({
      steps: Object.fromEntries(plan.steps.slice(1).map(step => [step, true]))
    })
  }), /truncated-playerdata-fallback/)
})
