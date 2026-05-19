import test from 'node:test'
import assert from 'node:assert/strict'
import {
  INVENTORY_PARITY_SCENARIOS,
  createInventoryParityPlan,
  recordInventoryParity,
  runInventoryParityScenario,
  summarizeInventoryParity
} from './inventory_parity_scenarios.mjs'
import { offlineUuid } from './runner.mjs'

for (const kind of Object.keys(INVENTORY_PARITY_SCENARIOS)) {
  test(`createInventoryParityPlan covers ${kind}`, () => {
    const plan = createInventoryParityPlan(kind)
    assert.equal(plan.name, `mineflayer-inventory-${kind.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)}`)
    assert.equal(plan.uuid, offlineUuid(plan.username))
    assert.deepEqual(plan.steps, INVENTORY_PARITY_SCENARIOS[kind])
  })

  test(`runInventoryParityScenario validates all ${kind} steps`, async () => {
    const result = await runInventoryParityScenario(kind, {
      probe: async plan => ({
        steps: Object.fromEntries(plan.steps.map(step => [step, true])),
        details: Object.fromEntries(plan.steps.map(step => [step, { comparedToVanilla: true }]))
      })
    })
    assert.equal(result.summary.ok, true)
  })
}

test('summarizeInventoryParity fails on missing evidence', () => {
  const plan = createInventoryParityPlan('windowLifecycle')
  const evidence = { timeline: [] }
  recordInventoryParity(evidence, plan.steps[0])
  assert.equal(summarizeInventoryParity(evidence, plan).ok, false)
})

test('runInventoryParityScenario reports missing packet trace validation', async () => {
  await assert.rejects(() => runInventoryParityScenario('packetTraceValidation', {
    probe: async plan => ({
      steps: Object.fromEntries(plan.steps.slice(0, -1).map(step => [step, true]))
    })
  }), /vanilla-trace-comparison/)
})
