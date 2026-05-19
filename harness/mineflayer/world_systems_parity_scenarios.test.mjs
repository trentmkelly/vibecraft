import test from 'node:test'
import assert from 'node:assert/strict'
import {
  WORLD_SYSTEMS_PARITY_SCENARIOS,
  createWorldSystemsParityPlan,
  recordWorldSystems,
  runWorldSystemsParityScenario,
  summarizeWorldSystems
} from './world_systems_parity_scenarios.mjs'

for (const kind of Object.keys(WORLD_SYSTEMS_PARITY_SCENARIOS)) {
  test(`createWorldSystemsParityPlan covers ${kind}`, () => {
    const plan = createWorldSystemsParityPlan(kind)
    assert.equal(plan.name, `mineflayer-world-systems-${kind.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)}`)
    assert.deepEqual(plan.steps, WORLD_SYSTEMS_PARITY_SCENARIOS[kind])
  })

  test(`runWorldSystemsParityScenario validates all ${kind} steps`, async () => {
    const result = await runWorldSystemsParityScenario(kind, {
      probe: async plan => ({
        steps: Object.fromEntries(plan.steps.map(step => [step, true])),
        details: Object.fromEntries(plan.steps.map(step => [step, { comparedToVanilla: true }]))
      })
    })
    assert.equal(result.summary.ok, true)
  })
}

test('summarizeWorldSystems fails with incomplete evidence', () => {
  const plan = createWorldSystemsParityPlan('worldBorder')
  const evidence = { timeline: [] }
  recordWorldSystems(evidence, plan.steps[0])
  assert.equal(summarizeWorldSystems(evidence, plan).ok, false)
})

test('runWorldSystemsParityScenario reports missing loot context evidence', async () => {
  await assert.rejects(() => runWorldSystemsParityScenario('lootContext', {
    probe: async plan => ({
      steps: Object.fromEntries(plan.steps.filter(step => step !== 'random-sequence-id').map(step => [step, true]))
    })
  }), /random-sequence-id/)
})
