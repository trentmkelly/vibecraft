import test from 'node:test'
import assert from 'node:assert/strict'
import {
  COMMAND_BLOCK_FUNCTION_SCENARIOS,
  createCommandBlockFunctionPlan,
  recordCommandBlockFunction,
  runCommandBlockFunctionScenario,
  summarizeCommandBlockFunction
} from './command_block_function_scenarios.mjs'

for (const kind of Object.keys(COMMAND_BLOCK_FUNCTION_SCENARIOS)) {
  test(`createCommandBlockFunctionPlan covers ${kind}`, () => {
    const plan = createCommandBlockFunctionPlan(kind)
    assert.equal(plan.name, `mineflayer-command-block-function-${kind.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)}`)
    assert.deepEqual(plan.steps, COMMAND_BLOCK_FUNCTION_SCENARIOS[kind])
    assert.equal(plan.serverProperties['enable-command-block'], 'true')
  })

  test(`runCommandBlockFunctionScenario validates all ${kind} steps`, async () => {
    const result = await runCommandBlockFunctionScenario(kind, {
      probe: async plan => ({
        steps: Object.fromEntries(plan.steps.map(step => [step, true])),
        details: Object.fromEntries(plan.steps.map(step => [step, { comparedToVanilla: true }]))
      })
    })
    assert.equal(result.summary.ok, true)
  })
}

test('summarizeCommandBlockFunction fails on incomplete function evidence', () => {
  const plan = createCommandBlockFunctionPlan('functions')
  const evidence = { timeline: [] }
  recordCommandBlockFunction(evidence, plan.steps[0])
  assert.equal(summarizeCommandBlockFunction(evidence, plan).ok, false)
})

test('runCommandBlockFunctionScenario reports missing macro parity evidence', async () => {
  await assert.rejects(() => runCommandBlockFunctionScenario('functions', {
    probe: async plan => ({
      steps: Object.fromEntries(plan.steps.filter(step => step !== 'macro-parity').map(step => [step, true]))
    })
  }), /macro-parity/)
})
