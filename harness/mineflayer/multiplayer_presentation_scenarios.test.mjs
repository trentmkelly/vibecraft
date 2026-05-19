import test from 'node:test'
import assert from 'node:assert/strict'
import {
  MULTIPLAYER_PRESENTATION_SCENARIOS,
  createMultiplayerPresentationPlan,
  recordMultiplayerPresentation,
  runMultiplayerPresentationScenario,
  summarizeMultiplayerPresentation
} from './multiplayer_presentation_scenarios.mjs'

for (const kind of Object.keys(MULTIPLAYER_PRESENTATION_SCENARIOS)) {
  test(`createMultiplayerPresentationPlan covers ${kind}`, () => {
    const plan = createMultiplayerPresentationPlan(kind)
    assert.equal(plan.name, `mineflayer-multiplayer-${kind.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)}`)
    assert.deepEqual(plan.steps, MULTIPLAYER_PRESENTATION_SCENARIOS[kind])
  })

  test(`runMultiplayerPresentationScenario validates all ${kind} steps`, async () => {
    const result = await runMultiplayerPresentationScenario(kind, {
      probe: async plan => ({
        steps: Object.fromEntries(plan.steps.map(step => [step, true])),
        details: Object.fromEntries(plan.steps.map(step => [step, { comparedToVanilla: true }]))
      })
    })
    assert.equal(result.summary.ok, true)
  })
}

test('summarizeMultiplayerPresentation fails with incomplete evidence', () => {
  const plan = createMultiplayerPresentationPlan('presentation')
  const evidence = { timeline: [] }
  recordMultiplayerPresentation(evidence, plan.steps[0])
  assert.equal(summarizeMultiplayerPresentation(evidence, plan).ok, false)
})

test('runMultiplayerPresentationScenario reports missing duplicate cleanup evidence', async () => {
  await assert.rejects(() => runMultiplayerPresentationScenario('duplicateSessionCleanup', {
    probe: async plan => ({
      steps: Object.fromEntries(plan.steps.filter(step => step !== 'playerdata-ownership').map(step => [step, true]))
    })
  }), /playerdata-ownership/)
})
