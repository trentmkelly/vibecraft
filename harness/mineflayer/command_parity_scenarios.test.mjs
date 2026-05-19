import test from 'node:test'
import assert from 'node:assert/strict'
import {
  COMMAND_PARITY_SCENARIOS,
  createCommandParityPlan,
  recordCommandParity,
  runCommandParityScenario,
  summarizeCommandParity
} from './command_parity_scenarios.mjs'

for (const kind of Object.keys(COMMAND_PARITY_SCENARIOS)) {
  test(`createCommandParityPlan covers ${kind}`, () => {
    const plan = createCommandParityPlan(kind)
    assert.equal(plan.name, `mineflayer-command-${kind.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)}`)
    assert.deepEqual(plan.steps, COMMAND_PARITY_SCENARIOS[kind])
  })

  test(`runCommandParityScenario validates all ${kind} steps`, async () => {
    const result = await runCommandParityScenario(kind, {
      probe: async plan => ({
        steps: Object.fromEntries(plan.steps.map(step => [step, true])),
        details: Object.fromEntries(plan.steps.map(step => [step, { comparedToVanilla: true }]))
      })
    })
    assert.equal(result.summary.ok, true)
  })
}

test('summarizeCommandParity fails when command evidence is incomplete', () => {
  const plan = createCommandParityPlan('suggestions')
  const evidence = { timeline: [] }
  recordCommandParity(evidence, plan.steps[0])
  assert.equal(summarizeCommandParity(evidence, plan).ok, false)
})

test('runCommandParityScenario reports missing command-block source evidence', async () => {
  await assert.rejects(() => runCommandParityScenario('resultConsistency', {
    probe: async plan => ({
      steps: Object.fromEntries(plan.steps.filter(step => step !== 'command-block-source').map(step => [step, true]))
    })
  }), /command-block-source/)
})
